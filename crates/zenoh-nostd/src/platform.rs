use ::core::net::SocketAddr;

use zenoh_proto::ZResult;

pub mod tcp;
pub mod ws;

pub trait ZPlatform {
    type ZTcpStream: tcp::ZTcpStream;
    type ZWsStream: ws::ZWsStream;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::ZTcpStream, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::ZWsStream, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }
}
