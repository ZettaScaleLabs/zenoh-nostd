use futures_lite::{AsyncReadExt, AsyncWriteExt};

use zenoh_nostd::{
    ZResult,
    platform::tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
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

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, zenoh_nostd::ZLinkError> {
        self.stream.write(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), zenoh_nostd::ZLinkError> {
        self.stream.write_all(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write_all ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, zenoh_nostd::ZLinkError> {
        self.stream.read(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );
            zenoh_nostd::ZLinkError::ReadOperationFailed
        })
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), zenoh_nostd::ZLinkError> {
        self.stream.read_exact(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read_exact ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );
            zenoh_nostd::ZLinkError::ReadOperationFailed
        })
    }
}

impl AbstractedTcpTx for StdTcpTx {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, zenoh_nostd::ZLinkError> {
        self.stream.write(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), zenoh_nostd::ZLinkError> {
        self.stream.write_all(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write_all ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }
}

impl AbstractedTcpRx for StdTcpRx {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, zenoh_nostd::ZLinkError> {
        self.stream.read(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), zenoh_nostd::ZLinkError> {
        self.stream.read_exact(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read_exact ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::ZLinkError::WriteOperationFailed
        })
    }
}
