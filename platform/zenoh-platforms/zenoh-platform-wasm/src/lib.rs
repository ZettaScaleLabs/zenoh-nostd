use core::{future::Future, time::Duration};

use async_wsocket::{Url, WebSocket};
use zenoh_platform::Platform;
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

pub mod ws;

pub struct PlatformWasm;

impl Platform for PlatformWasm {
    type PlatformTcpStream = zenoh_platform::tcp::DummyPlatformTcpStream;
    type PlatformWSStream = ws::PlatformWasmWSStream;

    fn new_ws_stream(
        &mut self,
        addr: &core::net::SocketAddr,
    ) -> impl Future<Output = ZResult<Self::PlatformWSStream>> {
        async move {
            let url = Url::parse(&format!("ws://{}", addr)).unwrap();

            let socket = WebSocket::connect(
                &url,
                &async_wsocket::ConnectionMode::Direct,
                Duration::from_secs(120),
            )
            .await
            .map_err(|_| zerr!(ZE::ConnectionRefused))
            .context(zctx!("WebSocket connection to address"))?;

            let peer_addr = *addr;

            Ok(Self::PlatformWSStream { peer_addr, socket })
        }
    }
}
