use core::net::SocketAddr;

use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Read, Write};
use zenoh_nostd::{
    platform::{ZCommunicationError, tcp::PALTcpStream},
    result::ZResult,
};

pub struct EmbassyTcpStream {
    pub socket: TcpSocket<'static>,

    pub local_addr: SocketAddr,
    pub peer_addr: SocketAddr,
}

impl PALTcpStream for EmbassyTcpStream {
    fn mtu(&self) -> u16 {
        1024
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
