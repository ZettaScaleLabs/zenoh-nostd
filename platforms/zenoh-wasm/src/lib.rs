use yawc::WebSocket;
use zenoh_nostd::platform::ZPlatform;

pub mod ws;

pub struct PlatformWasm;

impl ZPlatform for PlatformWasm {
    type TcpStream = zenoh_nostd::platform::tcp::DummyTcpStream;
    type WebSocket = ws::WasmWebSocket;

    async fn new_websocket_stream(
        &self,
        addr: &std::net::SocketAddr,
    ) -> core::result::Result<Self::WebSocket, zenoh_nostd::ConnectionError> {
        let url = format!("ws://{}", addr);
        let socket = WebSocket::connect(url.parse().map_err(|_| {
            zenoh_nostd::error!("Could not parse URL: {url}");
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?)
        .await
        .map_err(|_| {
            zenoh_nostd::error!("Could not connect to WebSocket");
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?;
        let peer_addr = *addr;
        Ok(Self::WebSocket::new(peer_addr, socket))
    }
}
