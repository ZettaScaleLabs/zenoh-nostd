use core::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;

use crate::{
    io::link::Link,
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

pub struct SingleLinkTransportMineConfig {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,

    pub keep_alive: usize,
    pub open_timeout: Duration,
}

pub struct SingleLinkTransportOtherConfig {
    pub other_whatami: WhatAmI,
    pub other_zid: ZenohIdProto,
    pub other_sn: TransportSn,
    pub other_lease: Duration,
}

pub struct SingleLinkTransportNegociatedConfig {
    pub mine_sn: TransportSn,

    pub resolution: Resolution,
    pub batch_size: BatchSize,
}

pub struct SingleLinkTransportConfig {
    pub mine_config: SingleLinkTransportMineConfig,
    pub other_config: SingleLinkTransportOtherConfig,
    pub negociated_config: SingleLinkTransportNegociatedConfig,
}

pub struct SingleLinkTransport<T: Platform> {
    link: Link<T>,
}

impl<T: Platform> SingleLinkTransport<T> {
    pub fn new(link: Link<T>) -> Self {
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

    pub async fn open<const TX: usize, const RX: usize>(
        link: Link<T>,
        config: SingleLinkTransportMineConfig,
    ) -> ZResult<(Self, SingleLinkTransportConfig), ZCommunicationError> {
        match select(
            Timer::after(config.open_timeout.try_into().unwrap()),
            async { establishment::open::open_link::<T, TX, RX>(link, config).await },
        )
        .await
        {
            embassy_futures::select::Either::First(_) => {
                zbail!(ZCommunicationError::TimedOut);
            }
            embassy_futures::select::Either::Second(res) => res,
        }
    }
}
