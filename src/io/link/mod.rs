use core::{net::SocketAddr, str::FromStr};

use crate::{
    io::link::{tcp::LinkTcp, ws::LinkWs},
    platform::{Platform, ZCommunicationError},
    protocol::core::endpoint::EndPoint,
    result::ZResult,
};

pub mod tcp;
pub mod ws;

pub enum Link<T: Platform> {
    LinkTcp(LinkTcp<T::PALTcpStream>),
    LinkWs(LinkWs<T::PALWebSocket>),
}

impl<T: Platform> Link<T> {
    pub fn is_reliable(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_reliable(),
            Self::LinkWs(ws) => ws.is_reliable(),
        }
    }

    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkTcp(tcp) => tcp.is_streamed(),
            Self::LinkWs(ws) => ws.is_streamed(),
        }
    }

    pub fn get_mtu(&self) -> u16 {
        match self {
            Self::LinkTcp(tcp) => tcp.get_mtu(),
            Self::LinkWs(ws) => ws.get_mtu(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write(buffer).await,
            Self::LinkWs(ws) => ws.write(buffer).await,
        }
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.write_all(buffer).await,
            Self::LinkWs(ws) => ws.write_all(buffer).await,
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read(buffer).await,
            Self::LinkWs(ws) => ws.read(buffer).await,
        }
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        match self {
            Self::LinkTcp(tcp) => tcp.read_exact(buffer).await,
            Self::LinkWs(ws) => ws.read_exact(buffer).await,
        }
    }

    pub async fn new(platform: &T, endpoint: EndPoint) -> ZResult<Self, ZCommunicationError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| ZCommunicationError::Invalid)?;

                let stream = platform.new_tcp_stream(&dst_addr).await?;

                Ok(Link::LinkTcp(LinkTcp::new(stream, dst_addr)))
            }
            "ws" => {
                let dst_addr = SocketAddr::from_str(address.as_str())
                    .map_err(|_| ZCommunicationError::Invalid)?;

                let ws = platform.new_websocket(&dst_addr).await?;

                Ok(Link::LinkWs(LinkWs::new(ws, dst_addr)))
            }
            _ => Err(ZCommunicationError::Invalid),
        }
    }
}
