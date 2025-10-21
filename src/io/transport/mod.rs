use core::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;

use crate::{
    io::{
        ZTransportError,
        link::{Link, LinkRx, LinkTx},
    },
    platform::Platform,
    protocol::{
        core::{ZenohIdProto, resolution::Resolution, whatami::WhatAmI},
        transport::{BatchSize, TransportMessage, TransportSn},
        zcodec::encode_array,
    },
    result::ZResult,
    zbail,
    zbuf::{BufWriterExt, ZBufExt, ZBufMut, ZBufMutExt, ZBufReader},
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
    pub(crate) other_sn: TransportSn,
    pub(crate) other_lease: Duration,
}

#[allow(dead_code)]
pub(crate) struct TransportNegociatedConfig {
    pub(crate) mine_sn: TransportSn,

    pub(crate) resolution: Resolution,
    pub(crate) batch_size: BatchSize,
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
        tx_zbuf: ZBufMut<'_>,
        rx_zbuf: ZBufMut<'_>,
    ) -> ZResult<(Self, TransportConfig), ZTransportError> {
        match select(
            Timer::after(config.open_timeout.try_into().unwrap()),
            async { establishment::open::open_link(link, config, tx_zbuf, rx_zbuf).await },
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
        mut tx_zbuf: ZBufMut<'_>,
        msg: &TransportMessage<'_, '_>,
    ) -> ZResult<(), ZTransportError> {
        let mut writer = tx_zbuf.writer();

        if self.link.is_streamed() {
            let space = BatchSize::MIN.to_le_bytes();
            encode_array(&mut writer, &space)?;
        }

        msg.encode(&mut writer)?;

        let remaining = writer.remaining();
        let space = BatchSize::MIN.to_le_bytes().len();
        let payload_len = (tx_zbuf.len() - space - remaining) as BatchSize;

        if self.link.is_streamed() {
            let len_bytes = payload_len.to_le_bytes();
            tx_zbuf[..space].copy_from_slice(&len_bytes);
        }

        self.link
            .write_all(&tx_zbuf[..payload_len as usize + space])
            .await
            .map_err(|e| e.into())
    }

    pub(crate) async fn recv<'a>(
        &mut self,
        rx_zbuf: ZBufMut<'a>,
    ) -> ZResult<ZBufReader<'a>, ZTransportError> {
        let n = if self.link.is_streamed() {
            let mut len = BatchSize::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = BatchSize::from_le_bytes(len) as usize;

            self.link.read_exact(&mut rx_zbuf[..l]).await?;

            l
        } else {
            self.link.read(rx_zbuf.as_mut()).await?
        };

        let slice: &'a [u8] = &rx_zbuf[..n];
        Ok(slice.reader())
    }
}

impl<'a, T: Platform> TransportTx<'a, T> {
    pub(crate) async fn send(
        &mut self,
        mut tx_zbuf: ZBufMut<'_>,
        msg: &TransportMessage<'_, '_>,
    ) -> ZResult<(), ZTransportError> {
        let mut writer = tx_zbuf.writer();

        if self.link.is_streamed() {
            let space = BatchSize::MIN.to_le_bytes();
            encode_array(&mut writer, &space)?;
        }

        msg.encode(&mut writer)?;

        let remaining = writer.remaining();
        let space = BatchSize::MIN.to_le_bytes().len();
        let payload_len = (tx_zbuf.len() - space - remaining) as BatchSize;

        if self.link.is_streamed() {
            let len_bytes = payload_len.to_le_bytes();
            tx_zbuf[..space].copy_from_slice(&len_bytes);
        }

        self.link
            .write_all(&tx_zbuf[..payload_len as usize + space])
            .await
            .map_err(|e| e.into())
    }
}

impl<'a, T: Platform> TransportRx<'a, T> {
    pub(crate) async fn recv<'b>(
        &mut self,
        rx_zbuf: ZBufMut<'b>,
    ) -> ZResult<ZBufReader<'b>, ZTransportError> {
        let n = if self.link.is_streamed() {
            let mut len = BatchSize::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = BatchSize::from_le_bytes(len) as usize;

            self.link.read_exact(&mut rx_zbuf[..l]).await?;

            l
        } else {
            self.link.read(rx_zbuf.as_mut()).await?
        };

        let slice: &'b [u8] = &rx_zbuf[..n];
        Ok(slice.reader())
    }
}
