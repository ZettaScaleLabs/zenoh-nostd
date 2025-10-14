use core::{net::SocketAddr, str::FromStr};

use crate::{
    io::link::tcp::{LinkTcp, LinkTcpRx, LinkTcpTx},
    platform::{Platform, ZCommunicationError},
    protocol::core::endpoint::EndPoint,
    result::ZResult,
};

pub mod tcp;

pub enum LinkTx<'a, T: Platform>
where
    T: 'a,
{
    LinkTcpTx(
        LinkTcpTx<<T::AbstractedTcpStream as crate::platform::tcp::AbstractedTcpStream>::Tx<'a>>,
    ),
}

pub enum LinkRx<'a, T: Platform>
where
    T: 'a,
{
    LinkTcpRx(
        LinkTcpRx<<T::AbstractedTcpStream as crate::platform::tcp::AbstractedTcpStream>::Rx<'a>>,
    ),
}

pub enum Link<T: Platform> {
    LinkTcp(LinkTcp<T::AbstractedTcpStream>),
}

impl<T: Platform> Link<T> {
    pub async fn new(platform: &T, endpoint: EndPoint) -> ZResult<Self, ZCommunicationError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| ZCommunicationError::Invalid)?;

                let stream = platform.new_tcp_stream(&dst_addr).await?;

                Ok(Self::LinkTcp(LinkTcp::new(stream)))
            }
            _ => Err(ZCommunicationError::Invalid),
        }
    }

    pub fn split(&mut self) -> (LinkTx<'_, T>, LinkRx<'_, T>) {
        match self {
            Self::LinkTcp(tcp) => {
                let (tx, rx) = tcp.split();
                (LinkTx::LinkTcpTx(tx), LinkRx::LinkTcpRx(rx))
            }
        }
    }

    pub fn is_reliable(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_reliable(),
        }
    }

    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_streamed(),
        }
    }

    pub fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcp(tcp) => tcp.mtu(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write(buffer).await,
        }
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write_all(buffer).await,
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read(buffer).await,
        }
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read_exact(buffer).await,
        }
    }
}

impl<T: Platform> LinkTx<'_, T> {
    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpTx(tcp) => tcp.is_streamed(),
        }
    }

    pub fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcpTx(tcp) => tcp.mtu(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write(buffer).await,
        }
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcpTx(tcp) => tcp.write_all(buffer).await,
        }
    }
}

impl<T: Platform> LinkRx<'_, T> {
    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcpRx(tcp) => tcp.is_streamed(),
        }
    }

    pub fn mtu(&self) -> u16 {
        match self {
            Self::LinkTcpRx(tcp) => tcp.mtu(),
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read(buffer).await,
        }
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcpRx(tcp) => tcp.read_exact(buffer).await,
        }
    }
}
