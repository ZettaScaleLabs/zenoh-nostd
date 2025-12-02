use embassy_futures::select::select;
use embassy_time::{Duration, Timer};
use zenoh_proto::{fields::*, *};

use crate::{
    io::link::{Link, LinkRx, LinkTx, ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::ZPlatform,
};

pub mod establishment;

#[derive(Clone)]
pub struct TransportMineConfig {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,

    pub keep_alive: usize,
    pub open_timeout: Duration,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TransportOtherConfig {
    pub other_whatami: WhatAmI,
    pub other_zid: ZenohIdProto,
    pub other_sn: u32,
    pub other_lease: Duration,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TransportNegociatedConfig {
    pub mine_sn: u32,

    pub resolution: Resolution,
    pub batch_size: u16,
}

#[derive(Clone)]
pub struct TransportConfig {
    pub mine_config: TransportMineConfig,
    pub other_config: TransportOtherConfig,
    pub negociated_config: TransportNegociatedConfig,
}

pub struct Transport<Platform>
where
    Platform: ZPlatform,
{
    link: Link<Platform>,
}

impl<Platform> Transport<Platform>
where
    Platform: ZPlatform,
{
    pub async fn open(
        link: Link<Platform>,
        config: TransportMineConfig,
        tx: &mut impl AsMut<[u8]>,
        rx: &mut impl AsMut<[u8]>,
    ) -> crate::ZResult<(Self, TransportConfig), crate::ZTransportError> {
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

    pub fn split(&mut self) -> (TransportTx<'_, Platform>, TransportRx<'_, Platform>) {
        let (link_tx, link_rx) = self.link.split();

        (TransportTx { tx: link_tx }, TransportRx { rx: link_rx })
    }
}

pub struct TransportTx<'a, Platform>
where
    Platform: ZPlatform,
{
    tx: LinkTx<'a, Platform>,
}

pub struct TransportRx<'a, Platform>
where
    Platform: ZPlatform,
{
    rx: LinkRx<'a, Platform>,
}

pub trait ZTransportTx {
    fn tx(&mut self) -> &mut impl ZLinkTx;

    fn send(
        &mut self,
        tx: &mut [u8],
        sn: &mut u32,
        mut writer: impl FnMut(&mut BatchWriter<&mut [u8]>) -> crate::ZResult<(), crate::ZCodecError>,
    ) -> impl ::core::future::Future<Output = crate::ZResult<(), crate::ZTransportError>> {
        let (mut batch, space) = if self.tx().is_streamed() {
            let space = u16::MIN.to_le_bytes();
            tx[..space.len()].copy_from_slice(&space);
            (BatchWriter::new(&mut tx[space.len()..], *sn), space.len())
        } else {
            (BatchWriter::new(&mut tx[..], *sn), 0)
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

pub trait ZTransportRx {
    fn rx(&mut self) -> &mut impl ZLinkRx;

    fn recv<'a>(
        &mut self,
        rx: &'a mut [u8],
    ) -> impl ::core::future::Future<Output = crate::ZResult<&'a [u8], crate::ZTransportError>>
    {
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

impl<Platform> ZTransportTx for TransportTx<'_, Platform>
where
    Platform: ZPlatform,
{
    fn tx(&mut self) -> &mut impl ZLinkTx {
        &mut self.tx
    }
}

impl<Platform> ZTransportRx for TransportRx<'_, Platform>
where
    Platform: ZPlatform,
{
    fn rx(&mut self) -> &mut impl ZLinkRx {
        &mut self.rx
    }
}

impl<Platform> ZTransportTx for Transport<Platform>
where
    Platform: ZPlatform,
{
    fn tx(&mut self) -> &mut impl ZLinkTx {
        &mut self.link
    }
}

impl<Platform> ZTransportRx for Transport<Platform>
where
    Platform: ZPlatform,
{
    fn rx(&mut self) -> &mut impl ZLinkRx {
        &mut self.link
    }
}
