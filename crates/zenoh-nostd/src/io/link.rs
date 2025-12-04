use core::{net::SocketAddr, str::FromStr};

use crate::{
    api::EndPoint,
    io::link::{
        tcp::{LinkTcp, LinkTcpRx, LinkTcpTx},
        ws::{LinkWs, LinkWsRx, LinkWsTx},
    },
    platform::{ZPlatform, tcp::ZTcpStream, ws::ZWebSocket},
};

pub mod tcp;
pub mod ws;

pub trait ZLinkInfo {
    fn mtu(&self) -> u16;

    fn is_streamed(&self) -> bool;
}

pub trait ZLinkTx: ZLinkInfo {
    #[allow(unused)]
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub trait ZLinkRx: ZLinkInfo {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub trait ZLink: ZLinkInfo + ZLinkTx + ZLinkRx {
    type Tx<'a>: ZLinkTx
    where
        Self: 'a;

    type Rx<'a>: ZLinkRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);
}

pub enum LinkTx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    LinkTcpTx(LinkTcpTx<<Platform::ZTcpStream as ZTcpStream>::Tx<'a>>),
    LinkWsTx(LinkWsTx<<Platform::ZWebSocket as ZWebSocket>::Tx<'a>>),
}

pub enum LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    LinkTcpRx(LinkTcpRx<<Platform::ZTcpStream as ZTcpStream>::Rx<'a>>),
    LinkWsRx(LinkWsRx<<Platform::ZWebSocket as ZWebSocket>::Rx<'a>>),
}

pub enum Link<Platform>
where
    Platform: ZPlatform,
{
    LinkTcp(LinkTcp<Platform::ZTcpStream>),
    LinkWs(LinkWs<Platform::ZWebSocket>),
}

impl<Platform> ZLinkInfo for Link<Platform>
where
    Platform: ZPlatform,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcp(tcp) => tcp.mtu(),
            Self::LinkWs(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_streamed(),
            Self::LinkWs(ws) => ws.is_streamed(),
        }
    }
}

impl<'a, Platform> ZLinkInfo for LinkTx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcpTx(tcp) => tcp.mtu(),
            Self::LinkWsTx(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpTx(tcp) => tcp.is_streamed(),
            Self::LinkWsTx(ws) => ws.is_streamed(),
        }
    }
}

impl<'a, Platform> ZLinkInfo for LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcpRx(tcp) => tcp.mtu(),
            Self::LinkWsRx(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpRx(tcp) => tcp.is_streamed(),
            Self::LinkWsRx(ws) => ws.is_streamed(),
        }
    }
}

impl<Platform> ZLinkTx for Link<Platform>
where
    Platform: ZPlatform,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write(buffer).await,
            Self::LinkWs(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write_all(buffer).await,
            Self::LinkWs(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<'a, Platform> ZLinkTx for LinkTx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write(buffer).await,
            Self::LinkWsTx(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write_all(buffer).await,
            Self::LinkWsTx(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<Platform> ZLinkRx for Link<Platform>
where
    Platform: ZPlatform,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read(buffer).await,
            Self::LinkWs(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read_exact(buffer).await,
            Self::LinkWs(ws) => ws.read_exact(buffer).await,
        }
    }
}

impl<'a, Platform> ZLinkRx for LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read(buffer).await,
            Self::LinkWsRx(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read_exact(buffer).await,
            Self::LinkWsRx(ws) => ws.read_exact(buffer).await,
        }
    }
}

impl<Platform> ZLink for Link<Platform>
where
    Platform: ZPlatform,
{
    type Tx<'a>
        = LinkTx<'a, Platform>
    where
        Self: 'a;

    type Rx<'a>
        = LinkRx<'a, Platform>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkTx<'_, Platform>, LinkRx<'_, Platform>) {
        match self {
            Self::LinkTcp(tcp) => {
                let (tx, rx) = tcp.split();
                (LinkTx::LinkTcpTx(tx), LinkRx::LinkTcpRx(rx))
            }
            Self::LinkWs(ws) => {
                let (tx, rx) = ws.split();
                (LinkTx::LinkWsTx(tx), LinkRx::LinkWsRx(rx))
            }
        }
    }
}

impl<Platform> Link<Platform>
where
    Platform: ZPlatform,
{
    pub(crate) async fn new(
        platform: &Platform,
        endpoint: EndPoint<'_>,
    ) -> core::result::Result<Self, crate::LinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let stream = platform.new_tcp_stream(&dst_addr).await?;

                Ok(Self::LinkTcp(LinkTcp::new(stream)))
            }
            "ws" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let stream = platform.new_websocket_stream(&dst_addr).await?;

                Ok(Self::LinkWs(LinkWs::new(stream)))
            }
            _ => Err(crate::EndpointError::CouldNotParseProtocol.into()),
        }
    }
}
