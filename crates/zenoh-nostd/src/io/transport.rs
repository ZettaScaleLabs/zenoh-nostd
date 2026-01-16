use embassy_time::{Duration, with_timeout};
use zenoh_proto::{
    exts::QoS,
    fields::*,
    msgs::{KeepAlive, NetworkBody, NetworkMessage, TransportMessage},
    *,
};
use zenoh_sansio::{Transport, TransportRx, TransportTx, ZTransportRx, ZTransportTx};

use crate::{
    ZConfig,
    io::link::{Link, LinkRx, LinkTx, ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
};

#[derive(Clone)]
pub struct TransportConfig {
    pub zid: ZenohIdProto,
    pub lease: Duration,
    pub resolution: Resolution,

    pub open_timeout: Duration,
}

pub struct TransportLink<Config>
where
    Config: ZConfig,
{
    link: Link<<Config as ZConfig>::Platform>,
    transport: Transport<<Config as ZConfig>::Buff>,
}

impl<Config> TransportLink<Config>
where
    Config: ZConfig,
{
    pub async fn connect(
        mut link: Link<<Config as ZConfig>::Platform>,
        config: TransportConfig,
        buff: <Config as ZConfig>::Buff,
    ) -> core::result::Result<Self, crate::TransportLinkError> {
        let connect = async move || {
            let transport = Transport::builder(buff)
                .with_zid(config.zid)
                .with_streamed(link.is_streamed())
                .with_lease(config.lease.into())
                .with_resolution(config.resolution)
                .connect_async(
                    &mut link,
                    async |link, bytes| {
                        if link.is_streamed() {
                            link.read_exact(bytes).await.map(|_| bytes.len())
                        } else {
                            link.read(bytes).await
                        }
                    },
                    async |link, bytes| link.write_all(bytes).await,
                )
                .await?
                .finish_async()
                .await?;

            Ok(Self { link, transport })
        };

        with_timeout(config.open_timeout, connect())
            .await
            .map_err(|_| TransportLinkError::OpenTimeout)
            .flatten()
    }

    pub async fn listen(
        mut link: Link<<Config as ZConfig>::Platform>,
        config: TransportConfig,
        buff: <Config as ZConfig>::Buff,
    ) -> core::result::Result<Self, crate::TransportLinkError> {
        let connect = async move || {
            let transport = Transport::builder(buff)
                .with_zid(config.zid)
                .with_streamed(link.is_streamed())
                .with_lease(config.lease.into())
                .with_resolution(config.resolution)
                .listen_async(
                    &mut link,
                    async |link, bytes| {
                        if link.is_streamed() {
                            link.read_exact(bytes).await.map(|_| bytes.len())
                        } else {
                            link.read(bytes).await
                        }
                    },
                    async |link, bytes| link.write_all(bytes).await,
                )
                .finish_async()
                .await?;

            Ok(Self { link, transport })
        };

        with_timeout(config.open_timeout, connect())
            .await
            .map_err(|_| TransportLinkError::OpenTimeout)
            .flatten()
    }

    pub fn split(&mut self) -> (TransportLinkTx<'_, Config>, TransportLinkRx<'_, Config>) {
        let (link_tx, link_rx) = self.link.split();

        (
            TransportLinkTx {
                link: link_tx,
                transport: &mut self.transport.tx,
            },
            TransportLinkRx {
                link: link_rx,
                transport: &mut self.transport.rx,
            },
        )
    }
}

pub struct TransportLinkTx<'a, Config>
where
    Config: ZConfig,
{
    pub link: LinkTx<'a, <Config as ZConfig>::Platform>,
    pub transport: &'a mut TransportTx<<Config as ZConfig>::Buff>,
}

pub struct TransportLinkRx<'a, Config>
where
    Config: ZConfig,
{
    pub link: LinkRx<'a, <Config as ZConfig>::Platform>,
    pub transport: &'a mut TransportRx<<Config as ZConfig>::Buff>,
}

pub trait ZTransportLinkTx {
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx);

    fn send<'a>(
        &mut self,
        msgs: impl Iterator<Item = NetworkBody<'a>>,
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::TransportLinkError>>
    {
        let (link, transport) = self.tx();
        transport.encode(msgs.map(|body| NetworkMessage {
            reliability: Reliability::Reliable,
            qos: QoS::default(),
            body: body,
        }));

        async move {
            if let Some(bytes) = transport.flush() {
                link.write_all(bytes).await.map_err(|e| e.into())
            } else {
                Ok(())
            }
        }
    }

    fn send_ref<'a>(
        &mut self,
        msgs: impl Iterator<Item = &'a NetworkMessage<'a>>,
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::TransportLinkError>>
    {
        let (link, transport) = self.tx();
        transport.encode_ref(msgs);

        async move {
            if let Some(bytes) = transport.flush() {
                link.write_all(bytes).await.map_err(|e| e.into())
            } else {
                Ok(())
            }
        }
    }

    fn keepalive(
        &mut self,
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::TransportLinkError>>
    {
        let (link, transport) = self.tx();
        transport.encode_t(core::iter::once(TransportMessage::KeepAlive(
            KeepAlive::default(),
        )));

        async move {
            if let Some(bytes) = transport.flush() {
                link.write_all(bytes).await.map_err(|e| e.into())
            } else {
                Ok(())
            }
        }
    }
}

pub trait ZTransportLinkRx {
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx);

    fn recv<'a>(
        &mut self,
    ) -> impl core::future::Future<
        Output = core::result::Result<
            impl Iterator<Item = NetworkMessage<'_>>,
            crate::TransportLinkError,
        >,
    > {
        let (link, transport) = self.rx();
        let streamed = link.is_streamed();

        async move {
            transport
                .decode_with_async(async |bytes| {
                    if streamed {
                        link.read_exact(bytes).await.map(|_| bytes.len())
                    } else {
                        link.read(bytes).await
                    }
                })
                .await?;

            Ok(transport.flush())
        }
    }
}

impl<Config> ZTransportLinkTx for TransportLinkTx<'_, Config>
where
    Config: ZConfig,
{
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx) {
        (&mut self.link, self.transport)
    }
}

impl<Config> ZTransportLinkRx for TransportLinkRx<'_, Config>
where
    Config: ZConfig,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, self.transport)
    }
}

impl<Config> ZTransportLinkTx for TransportLink<Config>
where
    Config: ZConfig,
{
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx) {
        (&mut self.link, &mut self.transport.tx)
    }
}

impl<Config> ZTransportLinkRx for TransportLink<Config>
where
    Config: ZConfig,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, &mut self.transport.rx)
    }
}
