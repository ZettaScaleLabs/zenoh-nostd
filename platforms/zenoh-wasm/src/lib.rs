use zenoh_nostd::{
    ZResult,
    platform::{Platform, ZConnectionError},
};

#[cfg(feature = "yawc")]
use yawc::WebSocket;
#[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
use {async_wsocket::WebSocket, core::time::Duration};

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

        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        let socket = WebSocket::connect(
            &url.parse().map_err(|_| {
                zenoh_nostd::error!("Could not parse URL: {url}");
                ZConnectionError::CouldNotConnect
            })?,
            &async_wsocket::ConnectionMode::Direct,
            Duration::from_secs(120),
        )
        .await
        .map_err(|_| {
            zenoh_nostd::error!("Could not connect to WebSocket");
            ZConnectionError::CouldNotConnect
        })?;

        #[cfg(feature = "yawc")]
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
