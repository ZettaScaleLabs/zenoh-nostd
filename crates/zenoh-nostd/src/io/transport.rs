use core::{net::SocketAddr, str::FromStr, time::Duration};

use embassy_time::with_timeout;
use zenoh_proto::{
    EndPoint, TransportLinkError,
    fields::{Resolution, ZenohIdProto},
};
use zenoh_sansio::{Transport, ZTransportRx, ZTransportTx};

use super::{Link, LinkRx, LinkTx, ZLink, ZLinkInfo, ZLinkManager, ZLinkRx, ZLinkTx};

mod rx;
mod traits;
mod tx;

pub use rx::*;
pub use traits::*;
pub use tx::*;

pub struct TransportLink<'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: Link<'a, LinkManager>,
    transport: Transport<Buff>,
}

impl<'a, LinkManager, Buff> TransportLink<'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(link: Link<'a, LinkManager>, transport: Transport<Buff>) -> Self {
        Self { link, transport }
    }

    pub fn split(
        &mut self,
    ) -> (
        TransportLinkTx<'_, 'a, LinkManager, Buff>,
        TransportLinkRx<'_, 'a, LinkManager, Buff>,
    ) {
        let (link_tx, link_rx) = self.link.split();
        let (transport_tx, transport_rx) = self.transport.split();

        (
            TransportLinkTx::new(link_tx, transport_tx),
            TransportLinkRx::new(link_rx, transport_rx),
        )
    }
}

impl<'a, LinkManager, Buff> ZTransportLinkTx for TransportLink<'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx) {
        (&mut self.link, &mut self.transport.tx)
    }
}

impl<'a, LinkManager, Buff> ZTransportLinkRx for TransportLink<'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, &mut self.transport.rx)
    }
}

pub struct TransportLinkManager<LinkManager> {
    link_manager: LinkManager,

    open_timeout: Duration,
    zid: ZenohIdProto,
    lease: Duration,
    resolution: Resolution,
}

impl<LinkManager> TransportLinkManager<LinkManager> {
    pub fn new(
        link_manager: LinkManager,
        open_timeout: Duration,
        zid: ZenohIdProto,
        lease: Duration,
        resolution: Resolution,
    ) -> Self {
        Self {
            link_manager,
            open_timeout,
            zid,
            lease,
            resolution,
        }
    }

    pub async fn connect<Buff>(
        &self,
        endpoint: EndPoint<'_>,
        buff: Buff,
    ) -> core::result::Result<TransportLink<'_, LinkManager, Buff>, TransportLinkError>
    where
        LinkManager: ZLinkManager,
        Buff: AsMut<[u8]> + AsRef<[u8]> + Clone,
    {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        let mut link = match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| zenoh_proto::EndpointError::CouldNotParseAddress)?;

                self.link_manager.connect_tcp(&dst_addr).await?
            }
            "udp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| zenoh_proto::EndpointError::CouldNotParseAddress)?;

                self.link_manager.connect_udp(&dst_addr).await?
            }
            _ => zenoh_proto::zbail!(zenoh_proto::EndpointError::CouldNotParseProtocol),
        };

        let connect = async || {
            let streamed = link.is_streamed();
            Transport::builder(buff)
                .with_zid(self.zid)
                .with_lease(self.lease)
                .with_resolution(self.resolution)
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
                .with_prefixed(streamed)
                .finish_async()
                .await
        };

        let transport = with_timeout(self.open_timeout.try_into().unwrap(), connect())
            .await
            .map_err(|_| TransportLinkError::OpenTimeout)??;

        Ok(TransportLink::new(link, transport))
    }

    pub async fn listen<Buff>(
        &self,
        endpoint: EndPoint<'_>,
        buff: Buff,
    ) -> core::result::Result<TransportLink<'_, LinkManager, Buff>, TransportLinkError>
    where
        LinkManager: ZLinkManager,
        Buff: AsMut<[u8]> + AsRef<[u8]> + Clone,
    {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        let mut link = match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| zenoh_proto::EndpointError::CouldNotParseAddress)?;

                self.link_manager.listen_tcp(&dst_addr).await?
            }
            "udp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| zenoh_proto::EndpointError::CouldNotParseAddress)?;

                self.link_manager.listen_udp(&dst_addr).await?
            }
            _ => zenoh_proto::zbail!(zenoh_proto::EndpointError::CouldNotParseProtocol),
        };

        let listen = async || {
            let streamed = link.is_streamed();
            Transport::builder(buff)
                .with_zid(self.zid)
                .with_lease(self.lease)
                .with_resolution(self.resolution)
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
                .with_prefixed(streamed)
                .finish_async()
                .await
        };

        let transport = with_timeout(self.open_timeout.try_into().unwrap(), listen())
            .await
            .map_err(|_| TransportLinkError::OpenTimeout)??;

        Ok(TransportLink::new(link, transport))
    }
}
