use zenoh_nostd::platform::udp::{ZUdpRx, ZUdpSocket, ZUdpTx};

pub struct StdUdpSocket {
    pub socket: async_net::UdpSocket,

    pub mtu: u16,
}

impl StdUdpSocket {
    pub fn new(socket: async_net::UdpSocket, mtu: u16) -> Self {
        Self { socket, mtu }
    }
}

pub struct StdUdpTx {
    pub socket: async_net::UdpSocket,
}

pub struct StdUdpRx {
    pub socket: async_net::UdpSocket,
}

impl ZUdpSocket for StdUdpSocket {
    type Tx<'a> = StdUdpTx;
    type Rx<'a> = StdUdpRx;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = StdUdpTx {
            socket: self.socket.clone(),
        };
        let rx = StdUdpRx {
            socket: self.socket.clone(),
        };
        (tx, rx)
    }
}

impl ZUdpTx for StdUdpSocket {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.send(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZUdpTx for StdUdpTx {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.send(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "write ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZUdpRx for StdUdpSocket {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.recv(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.read(buffer).await.map(|_| ())
    }
}

impl ZUdpRx for StdUdpRx {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket.recv(buffer).await.map_err(|e| {
            zenoh_nostd::error!(
                "read ({}:{}:{}) failed with buffer len {}: {:?}",
                file!(),
                line!(),
                column!(),
                buffer.len(),
                e
            );

            zenoh_nostd::LinkError::LinkTxFailed
        })
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.read(buffer).await.map(|_| ())
    }
}
