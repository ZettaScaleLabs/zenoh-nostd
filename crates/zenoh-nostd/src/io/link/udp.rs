use crate::{
    io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::udp::{ZUdpRx, ZUdpSocket, ZUdpTx},
};

pub struct LinkUdp<Socket> {
    socket: Socket,
    mtu: u16,
}

impl<Socket> LinkUdp<Socket>
where
    Socket: ZUdpSocket,
{
    pub fn new(socket: Socket) -> Self {
        let mtu = socket.mtu();

        Self { socket, mtu }
    }
}

impl<Socket> ZLinkInfo for LinkUdp<Socket>
where
    Socket: ZUdpSocket,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        false
    }
}

impl<Socket> ZLinkTx for LinkUdp<Socket>
where
    Socket: ZUdpSocket,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.socket.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.socket.write_all(buffer).await
    }
}

impl<Socket> ZLinkRx for LinkUdp<Socket>
where
    Socket: ZUdpSocket,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        self.socket.read(buffer).await
    }
    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        self.socket.read_exact(buffer).await
    }
}

impl<Socket> ZLink for LinkUdp<Socket>
where
    Socket: ZUdpSocket,
{
    type Tx<'a>
        = LinkUdpTx<Socket::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = LinkUdpRx<Socket::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkUdpTx<Socket::Tx<'_>>, LinkUdpRx<Socket::Rx<'_>>) {
        let (tx, rx) = self.socket.split();
        let tx = LinkUdpTx { tx, mtu: self.mtu };
        let rx = LinkUdpRx { rx, mtu: self.mtu };
        (tx, rx)
    }
}

pub struct LinkUdpTx<Tx> {
    tx: Tx,
    mtu: u16,
}

impl<Tx> ZLinkInfo for LinkUdpTx<Tx>
where
    Tx: ZUdpTx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        false
    }
}

impl<Tx> ZLinkTx for LinkUdpTx<Tx>
where
    Tx: ZUdpTx,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.tx.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.tx.write_all(buffer).await
    }
}

pub struct LinkUdpRx<Rx> {
    rx: Rx,
    mtu: u16,
}

impl<Rx> ZLinkInfo for LinkUdpRx<Rx>
where
    Rx: ZUdpRx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        false
    }
}

impl<Rx> ZLinkRx for LinkUdpRx<Rx>
where
    Rx: ZUdpRx,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        self.rx.read(buffer).await
    }
    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        self.rx.read_exact(buffer).await
    }
}
