use futures_lite::{AsyncReadExt, AsyncWriteExt};

use crate::{
    platform::{ZCommunicationError, tcp::PALTcpStream},
    result::ZResult,
};

pub struct StdTcpStream {
    pub socket: async_net::TcpStream,
    pub mtu: u16,
}

impl PALTcpStream for StdTcpStream {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.socket
            .write(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }
}
