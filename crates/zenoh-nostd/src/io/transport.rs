use core::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;
use zenoh_proto::{
    Resolution, WhatAmI, ZCodecError, ZResult, ZenohIdProto, transport::Batch, zbail,
};

use crate::{
    io::{
        ZTransportError,
        link::{Link, LinkRx, LinkTx},
    },
    platform::Platform,
};

pub(crate) mod establishment;

pub(crate) struct TransportMineConfig {
    pub(crate) mine_zid: ZenohIdProto,
    pub(crate) mine_lease: Duration,

    pub(crate) keep_alive: usize,
    pub(crate) open_timeout: Duration,
}

#[allow(dead_code)]
pub(crate) struct TransportOtherConfig {
    pub(crate) other_whatami: WhatAmI,
    pub(crate) other_zid: ZenohIdProto,
    pub(crate) other_sn: u32,
    pub(crate) other_lease: Duration,
}

#[allow(dead_code)]
pub(crate) struct TransportNegociatedConfig {
    pub(crate) mine_sn: u32,

    pub(crate) resolution: Resolution,
    pub(crate) batch_size: u16,
}

pub(crate) struct TransportConfig {
    pub(crate) mine_config: TransportMineConfig,
    pub(crate) other_config: TransportOtherConfig,
    pub(crate) negociated_config: TransportNegociatedConfig,
}

pub struct Transport<T: Platform> {
    link: Link<T>,
}

pub(crate) struct TransportTx<'a, T: Platform> {
    link: LinkTx<'a, T>,
}

pub(crate) struct TransportRx<'a, T: Platform> {
    link: LinkRx<'a, T>,
}

impl<T: Platform> Transport<T> {
    pub(crate) async fn open(
        link: Link<T>,
        config: TransportMineConfig,
        tx: &mut [u8],
        rx: &mut [u8],
    ) -> ZResult<(Self, TransportConfig), ZTransportError> {
        match select(
            Timer::after(config.open_timeout.try_into().unwrap()),
            async { establishment::open::open_link(link, config, tx, rx).await },
        )
        .await
        {
            embassy_futures::select::Either::First(_) => {
                zbail!(ZTransportError::Timeout);
            }
            embassy_futures::select::Either::Second(res) => res,
        }
    }

    pub(crate) fn split(&mut self) -> (TransportTx<'_, T>, TransportRx<'_, T>) {
        let (link_tx, link_rx) = self.link.split();

        (TransportTx { link: link_tx }, TransportRx { link: link_rx })
    }

    pub(crate) async fn send(
        &mut self,
        tx: &mut [u8],
        sn: &mut u32,
        mut writer: impl FnMut(&mut Batch) -> ZResult<(), ZCodecError>,
    ) -> ZResult<(), ZTransportError> {
        let (mut batch, space) = if self.link.is_streamed() {
            let space = u16::MIN.to_le_bytes();
            tx[..space.len()].copy_from_slice(&space);
            (Batch::new(&mut tx[space.len()..], *sn), space.len())
        } else {
            (Batch::new(tx, *sn), 0)
        };

        writer(&mut batch)?;

        let (next_sn, payload_len) = batch.finalize();
        *sn = next_sn;

        if self.link.is_streamed() {
            let len_bytes = (payload_len as u16).to_le_bytes();
            tx[..space].copy_from_slice(&len_bytes);
        }

        self.link
            .write_all(&tx[..payload_len + space])
            .await
            .map_err(|e| e.into())
    }

    pub(crate) async fn recv<'a>(
        &mut self,
        rx: &'a mut [u8],
    ) -> ZResult<&'a [u8], ZTransportError> {
        let n = if self.link.is_streamed() {
            let mut len = u16::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = u16::from_le_bytes(len) as usize;

            self.link.read_exact(&mut rx[..l]).await?;

            l
        } else {
            self.link.read(rx.as_mut()).await?
        };

        let slice: &'a [u8] = &rx[..n];

        Ok(slice)
    }
}

impl<'a, T: Platform> TransportTx<'a, T> {
    pub(crate) async fn send(
        &mut self,
        tx: &mut [u8],
        sn: &mut u32,
        mut writer: impl FnMut(&mut Batch) -> ZResult<(), ZCodecError>,
    ) -> ZResult<(), ZTransportError> {
        let (mut batch, space) = if self.link.is_streamed() {
            let space = u16::MIN.to_le_bytes();
            tx[..space.len()].copy_from_slice(&space);
            (Batch::new(&mut tx[space.len()..], *sn), space.len())
        } else {
            (Batch::new(tx, *sn), 0)
        };

        writer(&mut batch)?;

        let (next_sn, payload_len) = batch.finalize();
        *sn = next_sn;

        if self.link.is_streamed() {
            let len_bytes = (payload_len as u16).to_le_bytes();
            tx[..space].copy_from_slice(&len_bytes);
        }

        self.link
            .write_all(&tx[..payload_len + space])
            .await
            .map_err(|e| e.into())
    }
}

impl<'a, T: Platform> TransportRx<'a, T> {
    pub(crate) async fn recv<'b>(
        &mut self,
        rx: &'b mut [u8],
    ) -> ZResult<&'b [u8], ZTransportError> {
        let n = if self.link.is_streamed() {
            let mut len = u16::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = u16::from_le_bytes(len) as usize;

            self.link.read_exact(&mut rx[..l]).await?;

            l
        } else {
            self.link.read(rx.as_mut()).await?
        };

        let slice: &'b [u8] = &rx[..n];

        Ok(slice)
    }
}
