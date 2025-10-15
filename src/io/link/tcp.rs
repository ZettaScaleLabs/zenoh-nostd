use crate::{
    platform::{
        ZCommunicationError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
    result::ZResult,
};

pub(crate) struct LinkTcp<T: AbstractedTcpStream> {
    stream: T,

    mtu: u16,
}

pub(crate) struct LinkTcpTx<T: AbstractedTcpTx> {
    tx: T,

    mtu: u16,
}

pub(crate) struct LinkTcpRx<T: AbstractedTcpRx> {
    rx: T,

    mtu: u16,
}

impl<T: AbstractedTcpStream> LinkTcp<T> {
    pub(crate) fn new(stream: T) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }

    pub(crate) fn split(&mut self) -> (LinkTcpTx<T::Tx<'_>>, LinkTcpRx<T::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkTcpTx { tx, mtu: self.mtu };
        let rx = LinkTcpRx { rx, mtu: self.mtu };
        (tx, rx)
    }

    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }

    pub(crate) fn is_reliable(&self) -> bool {
        true
    }

    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream.write(buffer).await
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.stream.write_all(buffer).await
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream.read(buffer).await
    }

    pub(crate) async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> ZResult<(), ZCommunicationError> {
        self.stream.read_exact(buffer).await
    }
}

impl<T: AbstractedTcpTx> LinkTcpTx<T> {
    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }

    pub(crate) fn is_reliable(&self) -> bool {
        true
    }

    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.tx.write(buffer).await
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.tx.write_all(buffer).await
    }
}

impl<T: AbstractedTcpRx> LinkTcpRx<T> {
    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }

    pub(crate) fn is_reliable(&self) -> bool {
        true
    }

    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.rx.read(buffer).await
    }

    pub(crate) async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> ZResult<(), ZCommunicationError> {
        self.rx.read_exact(buffer).await
    }
}
