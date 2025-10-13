use futures_lite::{AsyncReadExt, AsyncWriteExt};

use crate::{
    platform::{
        ZCommunicationError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
    result::ZResult,
};

pub struct StdTcpStream {
    pub socket: async_net::TcpStream,

    pub mtu: u16,
}

impl StdTcpStream {
    pub fn new(socket: async_net::TcpStream, mtu: u16) -> Self {
        Self {
            socket,
            mtu,
        }
    }
}

pub struct StdTcpTx {
    pub socket: async_net::TcpStream,
}

pub struct StdTcpRx {
    pub socket: async_net::TcpStream,
}

impl AbstractedTcpStream for StdTcpStream {
    type Tx = StdTcpTx;
    type Rx = StdTcpRx;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(self) -> (Self::Tx, Self::Rx) {
        let tx = StdTcpTx {
            socket: self.socket.clone(),
        };
        let rx = StdTcpRx {
            socket: self.socket,
        };
        (tx, rx)
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

impl AbstractedTcpTx for StdTcpTx {
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
}

impl AbstractedTcpRx for StdTcpRx {
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
