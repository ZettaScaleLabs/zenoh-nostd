use crate::{
    io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::tcp::{ZTcpRx, ZTcpStream, ZTcpTx},
};

pub struct LinkTcp<Stream> {
    stream: Stream,
    mtu: u16,
}

impl<Stream> LinkTcp<Stream>
where
    Stream: ZTcpStream,
{
    pub fn new(stream: Stream) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }
}

impl<Stream> ZLinkInfo for LinkTcp<Stream>
where
    Stream: ZTcpStream,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Stream> ZLinkTx for LinkTcp<Stream>
where
    Stream: ZTcpStream,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.stream.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.stream.write_all(buffer).await
    }
}

impl<Stream> ZLinkRx for LinkTcp<Stream>
where
    Stream: ZTcpStream,
{
    async fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        self.stream.read(buffer).await
    }
    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        self.stream.read_exact(buffer).await
    }
}

impl<Stream> ZLink for LinkTcp<Stream>
where
    Stream: ZTcpStream,
{
    type Tx<'a>
        = LinkTcpTx<Stream::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = LinkTcpRx<Stream::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkTcpTx<Stream::Tx<'_>>, LinkTcpRx<Stream::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkTcpTx { tx, mtu: self.mtu };
        let rx = LinkTcpRx { rx, mtu: self.mtu };
        (tx, rx)
    }
}

pub struct LinkTcpTx<Tx> {
    tx: Tx,
    mtu: u16,
}

impl<Tx> ZLinkInfo for LinkTcpTx<Tx>
where
    Tx: ZTcpTx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Tx> ZLinkTx for LinkTcpTx<Tx>
where
    Tx: ZTcpTx,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.tx.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.tx.write_all(buffer).await
    }
}

pub struct LinkTcpRx<Rx> {
    rx: Rx,
    mtu: u16,
}

impl<Rx> ZLinkInfo for LinkTcpRx<Rx>
where
    Rx: ZTcpRx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Rx> ZLinkRx for LinkTcpRx<Rx>
where
    Rx: ZTcpRx,
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
