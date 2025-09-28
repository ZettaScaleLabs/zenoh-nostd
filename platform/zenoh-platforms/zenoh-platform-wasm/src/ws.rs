use core::net::SocketAddr;

use async_wsocket::{futures_util::SinkExt, Message, WebSocket};
use futures_lite::StreamExt;
use zenoh_platform::ws::PlatformWSStream;
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

pub struct PlatformWasmWSStream {
    pub peer_addr: SocketAddr,

    pub socket: WebSocket,
}

impl PlatformWSStream for PlatformWasmWSStream {
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
        self.socket
            .send(Message::Binary(buffer.to_vec()))
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))
            .context(zctx!("WebSocket write"))?;

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.write(buffer)
            .await
            .map(|_| ())
            .context(zctx!("WebSocket write_all"))
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| zerr!(ZE::DidntRead))?
            .map_err(|_| zerr!(ZE::DidntRead))
            .context(zctx!("WebSocket read"))?;

        if let Message::Binary(data) = msg {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Err(zerr!(ZE::DidntRead))
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        let read_bytes = self
            .read(buffer)
            .await
            .context(zctx!("WebSocket read_exact"))?;

        if read_bytes == buffer.len() {
            Ok(())
        } else {
            Err(zerr!(ZE::CapacityExceeded))
        }
    }
}
