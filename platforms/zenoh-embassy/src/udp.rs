use embassy_net::udp::{UdpMetadata, UdpSocket};
use zenoh_nostd::platform::udp::{ZUdpRx, ZUdpSocket, ZUdpTx};

pub struct EmbassyUdpSocket {
    socket: UdpSocket<'static>,
    addr: UdpMetadata,
    mtu: u16,
}

impl EmbassyUdpSocket {
    pub fn new(socket: UdpSocket<'static>, metadata: UdpMetadata, mtu: u16) -> Self {
        Self {
            socket,
            addr: metadata,
            mtu,
        }
    }
}

pub struct EmbassyUdpTx<'a> {
    socket: &'a UdpSocket<'static>,
    addr: UdpMetadata,
}

pub struct EmbassyUdpRx<'a> {
    socket: &'a UdpSocket<'static>,
}

impl ZUdpSocket for EmbassyUdpSocket {
    type Tx<'a> = EmbassyUdpTx<'a>;
    type Rx<'a> = EmbassyUdpRx<'a>;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = EmbassyUdpTx {
            socket: &self.socket,
            addr: self.addr,
        };
        let rx = EmbassyUdpRx {
            socket: &self.socket,
        };
        (tx, rx)
    }
}

impl ZUdpTx for EmbassyUdpSocket {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket
            .send_to(buffer, self.addr)
            .await
            .map_err(|e| {
                zenoh_nostd::error!("EmbassyUdpSocket write error: {:?}", e);
                zenoh_nostd::LinkError::LinkTxFailed
            })
            .map(|_| buffer.len())
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZUdpTx for EmbassyUdpTx<'_> {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket
            .send_to(buffer, self.addr)
            .await
            .map_err(|e| {
                zenoh_nostd::error!("EmbassyUdpSocket write error: {:?}", e);
                zenoh_nostd::LinkError::LinkTxFailed
            })
            .map(|_| buffer.len())
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZUdpRx for EmbassyUdpSocket {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|e| {
                zenoh_nostd::error!("EmbassyUdpSocket read error: {:?}", e);
                zenoh_nostd::LinkError::LinkRxFailed
            })
            .map(|s| s.0)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.read(buffer).await.map(|_| ())
    }
}

impl ZUdpRx for EmbassyUdpRx<'_> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|e| {
                zenoh_nostd::error!("EmbassyUdpSocket read error: {:?}", e);
                zenoh_nostd::LinkError::LinkRxFailed
            })
            .map(|s| s.0)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.read(buffer).await.map(|_| ())
    }
}
