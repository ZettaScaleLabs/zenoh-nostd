use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embedded_io_async::{Read, Write};
use zenoh_nostd::platform::tcp::{ZTcpRx, ZTcpStream, ZTcpTx};

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

impl ZTcpStream for EmbassyTcpStream {
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
}

impl ZTcpTx for EmbassyTcpStream {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.write(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpStream write error: {:?}", e);
            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.socket.write_all(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpStream write_all error: {:?}", e);
            zenoh_nostd::LinkError::LinkTxFailed
        })
    }
}

impl ZTcpTx for EmbassyTcpTx<'_> {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.write(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpTx write error: {:?}", e);
            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.socket.write_all(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpTx write_all error: {:?}", e);
            zenoh_nostd::LinkError::LinkTxFailed
        })
    }
}

impl ZTcpRx for EmbassyTcpStream {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.read(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpStream read error: {:?}", e);
            zenoh_nostd::LinkError::LinkRxFailed
        })
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.socket.read_exact(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpStream read_exact error: {:?}", e);
            zenoh_nostd::LinkError::LinkRxFailed
        })
    }
}

impl ZTcpRx for EmbassyTcpRx<'_> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.read(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpRx read error: {:?}", e);
            zenoh_nostd::LinkError::LinkRxFailed
        })
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.socket.read_exact(buffer).await.map_err(|e| {
            zenoh_nostd::error!("EmbassyTcpRx read_exact error: {:?}", e);
            zenoh_nostd::LinkError::LinkRxFailed
        })
    }
}
