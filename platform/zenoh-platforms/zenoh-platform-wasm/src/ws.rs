use core::net::SocketAddr;
use std::time::Duration;

use async_wsocket::{futures_util::SinkExt, Message, Url, WebSocket};
use futures_lite::StreamExt;
use zenoh_platform::ws::PlatformWSStream;
use zenoh_result::{zerr, ZResult, ZE};

pub struct PlatformWasmWSStream {
    peer_addr: SocketAddr,

    socket: WebSocket,
}

impl PlatformWSStream for PlatformWasmWSStream {
    async fn connect(dst_addr: &SocketAddr) -> ZResult<Self> {
        let url: Url = Url::parse(&format!("ws://{}", dst_addr)).unwrap();

        let socket = WebSocket::connect(
            &url,
            &async_wsocket::ConnectionMode::Direct,
            Duration::from_secs(120),
        )
        .await
        .map_err(|_| zerr!(ZE::ConnectionRefused))?;

        let peer_addr = *dst_addr;

        Ok(Self { peer_addr, socket })
    }

    fn mtu(&self) -> u16 {
        u16::MAX
    }

    fn local_addr(&self) -> ZResult<SocketAddr> {
        Err(zerr!(ZE::UnsupportedPlatform))
    }

    fn peer_addr(&self) -> ZResult<SocketAddr> {
        Ok(self.peer_addr)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        web_sys::console::log_1(&format!("WS write {} bytes", buffer.len()).into());

        self.socket
            .send(Message::Binary(buffer.to_vec()))
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))?;

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.write(buffer).await.map(|_| ())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        web_sys::console::log_1(&"WS read".into());

        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| zerr!(ZE::DidntRead))?
            .map_err(|_| zerr!(ZE::DidntRead))?;

        if let Message::Binary(data) = msg {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Err(zerr!(ZE::DidntRead))
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        let read_bytes = self.read(buffer).await?;

        if read_bytes == buffer.len() {
            Ok(())
        } else {
            Err(zerr!(ZE::CapacityExceeded))
        }
    }
}
