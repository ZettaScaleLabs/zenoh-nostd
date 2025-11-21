use {
    async_wsocket::{Url, WebSocket},
    core::time::Duration,
    zenoh_nostd::{
        ZResult,
        platform::{Platform, ZConnectionError},
    },
};

pub mod ws;

pub struct PlatformWasm;

impl Platform for PlatformWasm {
    type AbstractedTcpStream = zenoh_nostd::platform::tcp::DummyTcpStream;
    type AbstractedWsStream = ws::WasmWsStream;

    async fn new_tcp_stream(
        &self,
        _addr: &std::net::SocketAddr,
    ) -> ZResult<Self::AbstractedTcpStream, ZConnectionError> {
        Err(ZConnectionError::CouldNotConnect)
    }

    async fn new_websocket_stream(
        &self,
        addr: &std::net::SocketAddr,
    ) -> ZResult<Self::AbstractedWsStream, ZConnectionError> {
        let url = Url::parse(&format!("ws://{}", addr)).unwrap();

        let socket = WebSocket::connect(
            &url,
            &async_wsocket::ConnectionMode::Direct,
            Duration::from_secs(120),
        )
        .await
        .map_err(|_| ZConnectionError::CouldNotConnect)?;

        let peer_addr = *addr;

        Ok(Self::AbstractedWsStream::new(peer_addr, socket))
    }
}
