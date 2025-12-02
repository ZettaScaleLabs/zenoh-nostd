use core::{net::SocketAddr, str::FromStr};

use zenoh_proto::EndPoint;

use crate::{
    io::link::{
        tcp::{LinkTcp, LinkTcpRx, LinkTcpTx},
        ws::{LinkWs, LinkWsRx, LinkWsTx},
    },
    platform::ZPlatform,
};

pub(crate) mod macros;

pub(crate) mod tcp;
pub(crate) mod ws;

pub trait ZLinkInfo {
    fn mtu(&self) -> u16;

    fn is_streamed(&self) -> bool;
}

pub trait ZLinkTx: ZLinkInfo {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl ::core::future::Future<Output = crate::ZResult<usize, crate::ZLinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl ::core::future::Future<Output = crate::ZResult<(), crate::ZLinkError>>;
}

pub trait ZLinkRx: ZLinkInfo {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl ::core::future::Future<Output = crate::ZResult<usize, crate::ZLinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl ::core::future::Future<Output = crate::ZResult<(), crate::ZLinkError>>;
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

pub enum LinkTx<'a, T: ZPlatform>
where
    T: 'a,
{
    LinkTcpTx(LinkTcpTx<<T::ZTcpStream as crate::platform::tcp::ZTcpStream>::Tx<'a>>),
    LinkWsTx(LinkWsTx<<T::ZWsStream as crate::platform::ws::ZWsStream>::Tx<'a>>),
}

pub enum LinkRx<'a, T: ZPlatform>
where
    T: 'a,
{
    LinkTcpRx(LinkTcpRx<<T::ZTcpStream as crate::platform::tcp::ZTcpStream>::Rx<'a>>),
    LinkWsRx(LinkWsRx<<T::ZWsStream as crate::platform::ws::ZWsStream>::Rx<'a>>),
}

pub enum Link<T: ZPlatform> {
    LinkTcp(LinkTcp<T::ZTcpStream>),
    LinkWs(LinkWs<T::ZWsStream>),
}

impl<T: ZPlatform> ZLinkInfo for Link<T> {
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

impl<T: ZPlatform> ZLinkInfo for LinkTx<'_, T> {
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

impl<T: ZPlatform> ZLinkInfo for LinkRx<'_, T> {
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

impl<T: ZPlatform> ZLinkTx for Link<T> {
    async fn write(&mut self, buffer: &[u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write(buffer).await,
            Self::LinkWs(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> crate::ZResult<(), crate::ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write_all(buffer).await,
            Self::LinkWs(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<T: ZPlatform> ZLinkTx for LinkTx<'_, T> {
    async fn write(&mut self, buffer: &[u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write(buffer).await,
            Self::LinkWsTx(ws) => ws.write(buffer).await,
        }
    }

    async fn write_all(&mut self, buffer: &[u8]) -> crate::ZResult<(), crate::ZLinkError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write_all(buffer).await,
            Self::LinkWsTx(ws) => ws.write_all(buffer).await,
        }
    }
}

impl<T: ZPlatform> ZLinkRx for Link<T> {
    async fn read(&mut self, buffer: &mut [u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read(buffer).await,
            Self::LinkWs(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> crate::ZResult<(), crate::ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read_exact(buffer).await,
            Self::LinkWs(ws) => ws.read_exact(buffer).await,
        }
    }
}

impl<T: ZPlatform> ZLinkRx for LinkRx<'_, T> {
    async fn read(&mut self, buffer: &mut [u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read(buffer).await,
            Self::LinkWsRx(ws) => ws.read(buffer).await,
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> crate::ZResult<(), crate::ZLinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read_exact(buffer).await,
            Self::LinkWsRx(ws) => ws.read_exact(buffer).await,
        }
    }
}

impl<T: ZPlatform> ZLink for Link<T> {
    type Tx<'a>
        = LinkTx<'a, T>
    where
        Self: 'a;

    type Rx<'a>
        = LinkRx<'a, T>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkTx<'_, T>, LinkRx<'_, T>) {
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

impl<T: ZPlatform> Link<T> {
    pub(crate) async fn new(
        platform: &T,
        endpoint: EndPoint<'_>,
    ) -> crate::ZResult<Self, crate::ZLinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::ZEndpointError::CouldNotParseEndpoint)?;

                let stream = platform.new_tcp_stream(&dst_addr).await?;

                Ok(Self::LinkTcp(LinkTcp::new(stream)))
            }
            "ws" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| crate::ZEndpointError::CouldNotParseEndpoint)?;

                let stream = platform.new_websocket_stream(&dst_addr).await?;

                Ok(Self::LinkWs(LinkWs::new(stream)))
            }
            _ => Err(crate::ZConnectionError::CouldNotConnect.into()),
        }
    }
}
