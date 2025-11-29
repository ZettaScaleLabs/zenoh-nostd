use zenoh_proto::ZResult;

use crate::platform::tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx};

pub(crate) struct LinkTcp<T: AbstractedTcpStream> {
    stream: T,

    mtu: u16,
}

pub(crate) struct LinkTcpTx<T: AbstractedTcpTx> {
    tx: T,
}

pub(crate) struct LinkTcpRx<T: AbstractedTcpRx> {
    rx: T,
}

impl<T: AbstractedTcpStream> LinkTcp<T> {
    pub(crate) fn new(stream: T) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }

    pub(crate) fn split(&mut self) -> (LinkTcpTx<T::Tx<'_>>, LinkTcpRx<T::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkTcpTx { tx };
        let rx = LinkTcpRx { rx };
        (tx, rx)
    }

    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }

    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), crate::ZLinkError> {
        self.stream.write_all(buffer).await.map_err(|e| e)
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, crate::ZLinkError> {
        self.stream.read(buffer).await.map_err(|e| e)
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), crate::ZLinkError> {
        self.stream.read_exact(buffer).await.map_err(|e| e)
    }
}

impl<T: AbstractedTcpTx> LinkTcpTx<T> {
    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), crate::ZLinkError> {
        self.tx.write_all(buffer).await.map_err(|e| e)
    }
}

impl<T: AbstractedTcpRx> LinkTcpRx<T> {
    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, crate::ZLinkError> {
        self.rx.read(buffer).await.map_err(|e| e)
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), crate::ZLinkError> {
        self.rx.read_exact(buffer).await.map_err(|e| e)
    }
}
