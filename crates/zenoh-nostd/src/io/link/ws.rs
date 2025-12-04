use crate::{
    io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::ws::{ZWebSocket, ZWsRx, ZWsTx},
};

pub struct LinkWs<Stream> {
    stream: Stream,
    mtu: u16,
}

impl<Stream> LinkWs<Stream>
where
    Stream: ZWebSocket,
{
    pub fn new(stream: Stream) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }
}

impl<Stream> ZLinkInfo for LinkWs<Stream>
where
    Stream: ZWebSocket,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Stream> ZLinkTx for LinkWs<Stream>
where
    Stream: ZWebSocket,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.stream.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.stream.write_all(buffer).await
    }
}

impl<Stream> ZLinkRx for LinkWs<Stream>
where
    Stream: ZWebSocket,
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

impl<Stream> ZLink for LinkWs<Stream>
where
    Stream: ZWebSocket,
{
    type Tx<'a>
        = LinkWsTx<Stream::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = LinkWsRx<Stream::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkWsTx<Stream::Tx<'_>>, LinkWsRx<Stream::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkWsTx { tx, mtu: self.mtu };
        let rx = LinkWsRx { rx, mtu: self.mtu };
        (tx, rx)
    }
}

pub struct LinkWsTx<Tx> {
    tx: Tx,
    mtu: u16,
}

impl<Tx> ZLinkInfo for LinkWsTx<Tx>
where
    Tx: ZWsTx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Tx> ZLinkTx for LinkWsTx<Tx>
where
    Tx: ZWsTx,
{
    async fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        self.tx.write(buffer).await
    }
    async fn write_all(&mut self, buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        self.tx.write_all(buffer).await
    }
}

pub struct LinkWsRx<Rx> {
    rx: Rx,
    mtu: u16,
}

impl<Rx> ZLinkInfo for LinkWsRx<Rx>
where
    Rx: ZWsRx,
{
    fn mtu(&self) -> u16 {
        self.mtu
    }
    fn is_streamed(&self) -> bool {
        true
    }
}

impl<Rx> ZLinkRx for LinkWsRx<Rx>
where
    Rx: ZWsRx,
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
