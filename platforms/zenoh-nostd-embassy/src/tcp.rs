use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embedded_io_async::{Read, Write};
use zenoh_nostd::{
    platform::{
        ZCommunicationError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
    result::ZResult,
};

pub struct EmbassyTcpStream {
    pub socket: TcpSocket<'static>,

    pub mtu: u16,
}

impl EmbassyTcpStream {
    pub fn new(socket: TcpSocket<'static>, mtu: u16) -> Self {
        Self { socket, mtu }
    }
}

pub struct EmbassyTcpTx<'a> {
    pub socket: TcpWriter<'a>,
}

pub struct EmbassyTcpRx<'a> {
    pub socket: TcpReader<'a>,
}

impl AbstractedTcpStream for EmbassyTcpStream {
    type Tx<'a> = EmbassyTcpTx<'a>;
    type Rx<'a> = EmbassyTcpRx<'a>;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let (rx, tx) = self.socket.split();
        let tx = EmbassyTcpTx { socket: tx };
        let rx = EmbassyTcpRx { socket: rx };
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

impl AbstractedTcpTx for EmbassyTcpTx<'_> {
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

impl AbstractedTcpRx for EmbassyTcpRx<'_> {
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
