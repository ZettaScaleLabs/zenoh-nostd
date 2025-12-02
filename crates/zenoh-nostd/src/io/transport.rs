use embassy_futures::select::select;
use embassy_time::{Duration, Timer};
use zenoh_proto::{fields::*, *};

use crate::io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};

pub(crate) mod establishment;

#[derive(Clone)]
pub(crate) struct TransportMineConfig {
    pub(crate) mine_zid: ZenohIdProto,
    pub(crate) mine_lease: Duration,

    pub(crate) keep_alive: usize,
    pub(crate) open_timeout: Duration,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct TransportOtherConfig {
    pub(crate) other_whatami: WhatAmI,
    pub(crate) other_zid: ZenohIdProto,
    pub(crate) other_sn: u32,
    pub(crate) other_lease: Duration,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct TransportNegociatedConfig {
    pub(crate) mine_sn: u32,

    pub(crate) resolution: Resolution,
    pub(crate) batch_size: u16,
}

#[derive(Clone)]
pub(crate) struct TransportConfig {
    pub(crate) mine_config: TransportMineConfig,
    pub(crate) other_config: TransportOtherConfig,
    pub(crate) negociated_config: TransportNegociatedConfig,
}

pub struct Transport<T: ZLink> {
    link: T,
}

impl<T: ZLink> Transport<T> {
    pub(crate) async fn open(
        link: T,
        config: TransportMineConfig,
        tx: &mut impl AsMut<[u8]>,
        rx: &mut impl AsMut<[u8]>,
    ) -> ZResult<(Self, TransportConfig), crate::ZTransportError> {
        match select(
            Timer::after(config.open_timeout.try_into().unwrap()),
            async { establishment::open::open_link(link, config, tx, rx).await },
        )
        .await
        {
            embassy_futures::select::Either::First(_) => {
                zbail!(crate::ZTransportError::Timeout);
            }
            embassy_futures::select::Either::Second(res) => res,
        }
    }
}

pub struct TransportTx<T: ZLinkTx> {
    tx: T,
}

pub struct TransportRx<T: ZLinkRx> {
    rx: T,
}

pub trait ZTransport: ZTransportSend + ZTransportRecv {
    type Tx<'a>: ZTransportSend
    where
        Self: 'a;

    type Rx<'a>: ZTransportRecv
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);
}

impl<T: ZLink> ZTransport for Transport<T> {
    type Tx<'a>
        = TransportTx<T::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = TransportRx<T::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (TransportTx<T::Tx<'_>>, TransportRx<T::Rx<'_>>) {
        let (link_tx, link_rx) = self.link.split();

        (TransportTx { tx: link_tx }, TransportRx { rx: link_rx })
    }
}

pub trait ZTransportSend {
    fn tx(&mut self) -> &mut impl ZLinkTx;

    fn send(
        &mut self,
        tx: &mut [u8],
        sn: &mut u32,
        mut writer: impl FnMut(&mut ZBatchWriter<&mut [u8]>) -> ZResult<(), crate::ZCodecError>,
    ) -> impl ::core::future::Future<Output = ZResult<(), crate::ZTransportError>> {
        let (mut batch, space) = if self.tx().is_streamed() {
            let space = u16::MIN.to_le_bytes();
            tx[..space.len()].copy_from_slice(&space);
            (ZBatchWriter::new(&mut tx[space.len()..], *sn), space.len())
        } else {
            (ZBatchWriter::new(&mut tx[..], *sn), 0)
        };

        let res = writer(&mut batch);

        let (next_sn, payload_len) = batch.finalize();
        *sn = next_sn;

        if self.tx().is_streamed() {
            let len_bytes = (payload_len as u16).to_le_bytes();
            tx[..space].copy_from_slice(&len_bytes);
        }

        async move {
            res?;

            self.tx()
                .write_all(&tx[..payload_len + space])
                .await
                .map_err(|e| e.into())
        }
    }
}

pub trait ZTransportRecv {
    fn rx(&mut self) -> &mut impl ZLinkRx;

    fn recv<'a>(
        &mut self,
        rx: &'a mut [u8],
    ) -> impl ::core::future::Future<Output = ZResult<&'a [u8], crate::ZTransportError>> {
        async move {
            let n = if self.rx().is_streamed() {
                let mut len = u16::MIN.to_le_bytes();
                self.rx().read_exact(&mut len).await?;
                let l = u16::from_le_bytes(len) as usize;

                self.rx().read_exact(&mut rx[..l]).await?;

                l
            } else {
                self.rx().read(rx.as_mut()).await?
            };

            let slice: &'a [u8] = &rx[..n];

            Ok(slice)
        }
    }
}

impl<T: ZLinkTx> ZTransportSend for TransportTx<T> {
    fn tx(&mut self) -> &mut impl ZLinkTx {
        &mut self.tx
    }
}

impl<T: ZLinkRx> ZTransportRecv for TransportRx<T> {
    fn rx(&mut self) -> &mut impl ZLinkRx {
        &mut self.rx
    }
}

impl<T: ZLink> ZTransportSend for Transport<T> {
    fn tx(&mut self) -> &mut impl ZLinkTx {
        &mut self.link
    }
}

impl<T: ZLink> ZTransportRecv for Transport<T> {
    fn rx(&mut self) -> &mut impl ZLinkRx {
        &mut self.link
    }
}
