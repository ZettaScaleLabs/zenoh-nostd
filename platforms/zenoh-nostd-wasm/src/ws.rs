use core::net::SocketAddr;

use async_wsocket::{Message, WebSocket, futures_util::SinkExt};
use futures_lite::StreamExt;
use zenoh_nostd::{
    platform::{ZCommunicationError, ws::PALWebSocket},
    result::ZResult,
    zbail,
};

pub struct WasmWebSocket {
    pub peer_addr: SocketAddr,

    pub socket: WebSocket,
}

impl PALWebSocket for WasmWebSocket {
    fn mtu(&self) -> u16 {
        u16::MAX
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.socket
            .send(Message::Binary(buffer.to_vec()))
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)?;

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.write(buffer).await.map(|_| ())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        let msg = self
            .socket
            .next()
            .await
            .map(|e| e.map_err(|_| ZCommunicationError::ConnectionClosed));

        if let Some(Ok(Message::Binary(data))) = msg {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            zbail!(ZCommunicationError::DidNotRead)
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        let msg = self
            .socket
            .next()
            .await
            .map(|e| e.map_err(|_| ZCommunicationError::ConnectionClosed));

        if let Some(Ok(Message::Binary(data))) = msg {
            if data.len() != buffer.len() {
                zbail!(ZCommunicationError::DidNotRead);
            }
            buffer.copy_from_slice(&data);
            Ok(())
        } else {
            zbail!(ZCommunicationError::DidNotRead)
        }
    }
}
