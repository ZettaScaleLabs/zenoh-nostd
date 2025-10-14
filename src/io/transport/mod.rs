use core::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;

use crate::{
    io::link::{Link, LinkRx, LinkTx},
    platform::{Platform, ZCommunicationError},
    protocol::{
        core::{ZenohIdProto, resolution::Resolution, whatami::WhatAmI},
        transport::{BatchSize, TransportMessage, TransportSn},
        zcodec::encode_array,
    },
    result::ZResult,
    zbail,
    zbuf::{BufWriterExt, ZBufExt, ZBufMut, ZBufMutExt, ZBufReader},
};

pub mod establishment;

pub struct TransportMineConfig {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,

    pub keep_alive: usize,
    pub open_timeout: Duration,
}

pub struct TransportOtherConfig {
    pub other_whatami: WhatAmI,
    pub other_zid: ZenohIdProto,
    pub other_sn: TransportSn,
    pub other_lease: Duration,
}

pub struct TransportNegociatedConfig {
    pub mine_sn: TransportSn,

    pub resolution: Resolution,
    pub batch_size: BatchSize,
}

pub struct TransportConfig {
    pub mine_config: TransportMineConfig,
    pub other_config: TransportOtherConfig,
    pub negociated_config: TransportNegociatedConfig,
}

pub struct Transport<T: Platform> {
    link: Link<T>,
}

pub struct TransportTx<'a, T: Platform> {
    link: LinkTx<'a, T>,
}

pub struct TransportRx<'a, T: Platform> {
    link: LinkRx<'a, T>,
}

impl<T: Platform> Transport<T> {
    pub async fn open(
        link: Link<T>,
        config: TransportMineConfig,
        tx_zbuf: ZBufMut<'_>,
        rx_zbuf: ZBufMut<'_>,
    ) -> ZResult<(Self, TransportConfig), ZCommunicationError> {
        match select(
            Timer::after(config.open_timeout.try_into().unwrap()),
            async { establishment::open::open_link(link, config, tx_zbuf, rx_zbuf).await },
        )
        .await
        {
            embassy_futures::select::Either::First(_) => {
                zbail!(ZCommunicationError::TimedOut);
            }
            embassy_futures::select::Either::Second(res) => res,
        }
    }

    pub fn split(&mut self) -> (TransportTx<'_, T>, TransportRx<'_, T>) {
        let (link_tx, link_rx) = self.link.split();

        (TransportTx { link: link_tx }, TransportRx { link: link_rx })
    }

    pub async fn send(
        &mut self,
        mut tx_zbuf: ZBufMut<'_>,
        msg: &TransportMessage<'_, '_>,
    ) -> ZResult<(), ZCommunicationError> {
        let mut writer = tx_zbuf.writer();

        if self.link.is_streamed() {
            let space = BatchSize::MIN.to_le_bytes();
            encode_array(&space, &mut writer)?;
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
    }

    pub async fn recv<'a>(
        &mut self,
        rx_zbuf: ZBufMut<'a>,
    ) -> ZResult<ZBufReader<'a>, ZCommunicationError> {
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
    pub fn new(link: LinkTx<'a, T>) -> Self {
        Self { link }
    }

    pub async fn send(
        &mut self,
        mut tx_zbuf: ZBufMut<'_>,
        msg: &TransportMessage<'_, '_>,
    ) -> ZResult<(), ZCommunicationError> {
        let mut writer = tx_zbuf.writer();

        if self.link.is_streamed() {
            let space = BatchSize::MIN.to_le_bytes();
            encode_array(&space, &mut writer)?;
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
    }
}

impl<'a, T: Platform> TransportRx<'a, T> {
    pub fn new(link: LinkRx<'a, T>) -> Self {
        Self { link }
    }

    pub async fn recv<'b>(
        &mut self,
        rx_zbuf: ZBufMut<'b>,
    ) -> ZResult<ZBufReader<'b>, ZCommunicationError> {
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
