use {
    yawc::WebSocket,
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
        let url = format!("ws://{}", addr);
        let socket = WebSocket::connect(url.parse().map_err(|_| {
            zenoh_nostd::error!("Could not parse URL: {url}");
            ZConnectionError::CouldNotConnect
        })?)
        .await
        .map_err(|_| {
            zenoh_nostd::error!("Could not connect to WebSocket");
            ZConnectionError::CouldNotConnect
        })?;
        let peer_addr = *addr;
        Ok(Self::AbstractedWsStream::new(peer_addr, socket))
    }
}
