use core::time::Duration;

use async_wsocket::{Url, WebSocket};
use zenoh_nostd::platform::{Platform, ZCommunicationError};

pub mod ws;

pub struct PlatformWasm;

impl Platform for PlatformWasm {
    type PALTcpStream = zenoh_nostd::platform::tcp::DummyTcpStream;
    type PALWebSocket = ws::WasmWebSocket;

    async fn new_tcp_stream(
        &self,
        _addr: &std::net::SocketAddr,
    ) -> zenoh_nostd::result::ZResult<
            Self::PALTcpStream,
            zenoh_nostd::platform::ZCommunicationError,
        > { Err(ZCommunicationError::Invalid) }

    async fn new_websocket(
        &self,
        addr: &std::net::SocketAddr,
    ) -> zenoh_nostd::result::ZResult<Self::PALWebSocket, ZCommunicationError>
    {
        let url = Url::parse(&format!("ws://{}", addr)).unwrap();

        let socket = WebSocket::connect(
            &url,
            &async_wsocket::ConnectionMode::Direct,
            Duration::from_secs(120),
        )
        .await
        .map_err(|_| ZCommunicationError::ConnectionClosed)?;

        let peer_addr = *addr;

        Ok(Self::PALWebSocket { peer_addr, socket })
    }
}
