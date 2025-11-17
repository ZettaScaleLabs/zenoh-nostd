use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embedded_io_async::{Read, Write};
use zenoh_nostd::{
    ZResult,
    platform::{
        ZConnectionError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
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

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.socket
            .write(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotRead)
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotRead)
    }
}

impl AbstractedTcpTx for EmbassyTcpTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.socket
            .write(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }
}

impl AbstractedTcpRx for EmbassyTcpRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotRead)
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotRead)
    }
}
