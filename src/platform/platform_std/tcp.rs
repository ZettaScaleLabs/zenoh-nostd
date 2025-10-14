use futures_lite::{AsyncReadExt, AsyncWriteExt};

use crate::{
    platform::{
        ZCommunicationError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
    result::ZResult,
};

pub struct StdTcpStream {
    pub stream: async_net::TcpStream,

    pub mtu: u16,
}

impl StdTcpStream {
    pub fn new(stream: async_net::TcpStream, mtu: u16) -> Self {
        Self { stream, mtu }
    }
}

pub struct StdTcpTx {
    pub stream: async_net::TcpStream,
}

pub struct StdTcpRx {
    pub stream: async_net::TcpStream,
}

impl AbstractedTcpStream for StdTcpStream {
    type Tx<'a> = StdTcpTx;
    type Rx<'a> = StdTcpRx;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = StdTcpTx {
            stream: self.stream.clone(),
        };
        let rx = StdTcpRx {
            stream: self.stream.clone(),
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream
            .write(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.stream
            .write_all(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream
            .read(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.stream
            .read_exact(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }
}

impl AbstractedTcpTx for StdTcpTx {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream
            .write(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.stream
            .write_all(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotWrite)
    }
}

impl AbstractedTcpRx for StdTcpRx {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream
            .read(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.stream
            .read_exact(buffer)
            .await
            .map_err(|_| ZCommunicationError::DidNotRead)
    }
}
