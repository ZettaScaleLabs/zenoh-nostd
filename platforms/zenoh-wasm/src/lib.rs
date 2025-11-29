use {yawc::WebSocket, zenoh_nostd::platform::Platform};

pub mod ws;

pub struct PlatformWasm;

impl Platform for PlatformWasm {
    type AbstractedTcpStream = zenoh_nostd::platform::tcp::DummyTcpStream;
    type AbstractedWsStream = ws::WasmWsStream;

    async fn new_websocket_stream(
        &self,
        addr: &std::net::SocketAddr,
    ) -> zenoh_nostd::ZResult<Self::AbstractedWsStream, zenoh_nostd::ZConnectionError> {
        let url = format!("ws://{}", addr);
        let socket = WebSocket::connect(url.parse().map_err(|_| {
            zenoh_nostd::error!("Could not parse URL: {url}");
            zenoh_nostd::ZConnectionError::CouldNotConnect
        })?)
        .await
        .map_err(|_| {
            zenoh_nostd::error!("Could not connect to WebSocket");
            zenoh_nostd::ZConnectionError::CouldNotConnect
        })?;
        let peer_addr = *addr;
        Ok(Self::AbstractedWsStream::new(peer_addr, socket))
    }
}
