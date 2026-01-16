use core::{net::SocketAddr, str::FromStr};

use crate::{
    EndPoint,
    io::link::{
        tcp::{LinkTcp, LinkTcpRx, LinkTcpTx},
        udp::{LinkUdp, LinkUdpRx, LinkUdpTx},
        ws::{LinkWs, LinkWsRx, LinkWsTx},
    },
    platform::{ZPlatform, tcp::ZTcpStream, udp::ZUdpSocket, ws::ZWebSocket},
};

mod tcp;
mod udp;
mod ws;

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
    Tcp(LinkTcpTx<<Platform::TcpStream as ZTcpStream>::Tx<'a>>),
    Udp(LinkUdpTx<<Platform::UdpSocket as ZUdpSocket>::Tx<'a>>),
    Ws(LinkWsTx<<Platform::WebSocket as ZWebSocket>::Tx<'a>>),
}

pub enum LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    Tcp(LinkTcpRx<<Platform::TcpStream as ZTcpStream>::Rx<'a>>),
    Udp(LinkUdpRx<<Platform::UdpSocket as ZUdpSocket>::Rx<'a>>),
    Ws(LinkWsRx<<Platform::WebSocket as ZWebSocket>::Rx<'a>>),
}

pub enum Link<Platform>
where
    Platform: ZPlatform,
{
    Tcp(LinkTcp<Platform::TcpStream>),
    Udp(LinkUdp<Platform::UdpSocket>),
    Ws(LinkWs<Platform::WebSocket>),
}

impl<Platform> ZLinkInfo for Link<Platform>
where
    Platform: ZPlatform,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::Tcp(tcp) => tcp.mtu(),
            Self::Udp(udp) => udp.mtu(),
            Self::Ws(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::Tcp(tcp) => tcp.is_streamed(),
            Self::Udp(udp) => udp.is_streamed(),
            Self::Ws(ws) => ws.is_streamed(),
        }
    }
}

impl<'a, Platform> ZLinkInfo for LinkTx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::Tcp(tcp) => tcp.mtu(),
            Self::Udp(udp) => udp.mtu(),
            Self::Ws(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::Tcp(tcp) => tcp.is_streamed(),
            Self::Udp(udp) => udp.is_streamed(),
            Self::Ws(ws) => ws.is_streamed(),
        }
    }
}

impl<'a, Platform> ZLinkInfo for LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    fn mtu(&self) -> u16 {
        match self {
            Self::Tcp(tcp) => tcp.mtu(),
            Self::Udp(udp) => udp.mtu(),
            Self::Ws(ws) => ws.mtu(),
        }
    }

    fn is_streamed(&self) -> bool {
        match self {
            Self::Tcp(tcp) => tcp.is_streamed(),
            Self::Udp(udp) => udp.is_streamed(),
            Self::Ws(ws) => ws.is_streamed(),
        }
    }
}

impl<Platform> ZLinkTx for Link<Platform>
where
    Platform: ZPlatform,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.write(buffer).await,
            Self::Udp(udp) => udp.write(buffer).await,
            Self::Ws(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.write_all(buffer).await,
            Self::Udp(udp) => udp.write_all(buffer).await,
            Self::Ws(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<'a, Platform> ZLinkTx for LinkTx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.write(buffer).await,
            Self::Udp(udp) => udp.write(buffer).await,
            Self::Ws(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.write_all(buffer).await,
            Self::Udp(udp) => udp.write_all(buffer).await,
            Self::Ws(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<Platform> ZLinkRx for Link<Platform>
where
    Platform: ZPlatform,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.read(buffer).await,
            Self::Udp(udp) => udp.read(buffer).await,
            Self::Ws(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.read_exact(buffer).await,
            Self::Udp(udp) => udp.read_exact(buffer).await,
            Self::Ws(ws) => ws.read_exact(buffer).await,
        }
    }
}

impl<'a, Platform> ZLinkRx for LinkRx<'a, Platform>
where
    Platform: ZPlatform + 'a,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.read(buffer).await,
            Self::Udp(udp) => udp.read(buffer).await,
            Self::Ws(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        match self {
            Self::Tcp(tcp) => tcp.read_exact(buffer).await,
            Self::Udp(udp) => udp.read_exact(buffer).await,
            Self::Ws(ws) => ws.read_exact(buffer).await,
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
            Self::Tcp(tcp) => {
                let (tx, rx) = tcp.split();
                (LinkTx::Tcp(tx), LinkRx::Tcp(rx))
            }
            Self::Udp(udp) => {
                let (tx, rx) = udp.split();
                (LinkTx::Udp(tx), LinkRx::Udp(rx))
            }
            Self::Ws(ws) => {
                let (tx, rx) = ws.split();
                (LinkTx::Ws(tx), LinkRx::Ws(rx))
            }
        }
    }
}

impl<Platform> Link<Platform>
where
    Platform: ZPlatform,
{
    pub(crate) async fn connect(
        platform: &Platform,
        endpoint: EndPoint<'_>,
    ) -> core::result::Result<Self, crate::LinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let stream = platform.connect_tcp(&dst_addr).await?;

                Ok(Self::Tcp(LinkTcp::new(stream)))
            }
            "udp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let socket = platform.connect_udp(&dst_addr).await?;

                Ok(Self::Udp(LinkUdp::new(socket)))
            }
            "ws" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let stream = platform.connect_websocket(&dst_addr).await?;

                Ok(Self::Ws(LinkWs::new(stream)))
            }
            _ => Err(crate::EndpointError::CouldNotParseProtocol.into()),
        }
    }

    pub(crate) async fn listen(
        platform: &Platform,
        endpoint: EndPoint<'_>,
    ) -> core::result::Result<Self, crate::LinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::EndpointError::CouldNotParseAddress)?;

                let stream = platform.listen_tcp(&dst_addr).await?;

                Ok(Self::Tcp(LinkTcp::new(stream)))
            }
            _ => Err(crate::EndpointError::CouldNotParseProtocol.into()),
        }
    }
}
