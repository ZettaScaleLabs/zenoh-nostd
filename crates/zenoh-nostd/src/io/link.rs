use core::{net::SocketAddr, str::FromStr};

use zenoh_proto::{EndPoint, ZResult};

use crate::{
    io::{
        ZLinkError,
        link::tcp::{LinkTcp, LinkTcpRx, LinkTcpTx},
    },
    platform::Platform,
};

pub(crate) mod tcp;

pub(crate) enum LinkTx<'a, T: Platform>
where
    T: 'a,
{
    LinkTcpTx(
        LinkTcpTx<<T::AbstractedTcpStream as crate::platform::tcp::AbstractedTcpStream>::Tx<'a>>,
    ),
}

pub(crate) enum LinkRx<'a, T: Platform>
where
    T: 'a,
{
    LinkTcpRx(
        LinkTcpRx<<T::AbstractedTcpStream as crate::platform::tcp::AbstractedTcpStream>::Rx<'a>>,
    ),
}

pub(crate) enum Link<T: Platform> {
    LinkTcp(LinkTcp<T::AbstractedTcpStream>),
}

impl<T: Platform> Link<T> {
    pub(crate) async fn new(platform: &T, endpoint: EndPoint) -> ZResult<Self, ZLinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| ZLinkError::CouldNotParse)?;

                let stream = platform.new_tcp_stream(&dst_addr).await?;

                Ok(Self::LinkTcp(LinkTcp::new(stream)))
            }
            _ => Err(ZLinkError::CouldNotConnect),
        }
    }

    pub(crate) fn split(&mut self) -> (LinkTx<'_, T>, LinkRx<'_, T>) {
        match self {
            Self::LinkTcp(tcp) => {
                let (tx, rx) = tcp.split();
                (LinkTx::LinkTcpTx(tx), LinkRx::LinkTcpRx(rx))
            }
        }
    }

    pub(crate) fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_streamed(),
        }
    }

    pub(crate) fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcp(tcp) => tcp.mtu(),
        }
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write_all(buffer).await,
        }
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read(buffer).await,
        }
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZLinkError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read_exact(buffer).await,
        }
    }
}

impl<T: Platform> LinkTx<'_, T> {
    pub(crate) fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpTx(tcp) => tcp.is_streamed(),
        }
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZLinkError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write_all(buffer).await,
        }
    }
}

impl<T: Platform> LinkRx<'_, T> {
    pub(crate) fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpRx(tcp) => tcp.is_streamed(),
        }
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZLinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read(buffer).await,
        }
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZLinkError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read_exact(buffer).await,
        }
    }
}
