use crate::{
    platform::{
        ZCommunicationError,
        tcp::{AbstractedTcpRx, AbstractedTcpStream, AbstractedTcpTx},
    },
    result::ZResult,
};

pub struct LinkTcp<T: AbstractedTcpStream> {
    tx: LinkTcpTx<T::Tx>,
    rx: LinkTcpRx<T::Rx>,

    mtu: u16,
}

pub struct LinkTcpTx<T: AbstractedTcpTx> {
    tx: T,

    mtu: u16,
}

pub struct LinkTcpRx<T: AbstractedTcpRx> {
    rx: T,

    mtu: u16,
}

impl<T: AbstractedTcpStream> LinkTcp<T> {
    pub fn new(stream: T) -> Self {
        let mtu = stream.mtu();
        let (tx, rx) = stream.split();

        Self {
            tx: LinkTcpTx { tx, mtu },
            rx: LinkTcpRx { rx, mtu },
            mtu,
        }
    }

    pub fn split(self) -> (LinkTcpTx<T::Tx>, LinkTcpRx<T::Rx>) {
        (self.tx, self.rx)
    }

    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn is_reliable(&self) -> bool {
        true
    }

    pub fn is_streamed(&self) -> bool {
        true
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.tx.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.tx.write_all(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.rx.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.rx.read_exact(buffer).await
    }
}

impl<T: AbstractedTcpTx> LinkTcpTx<T> {
    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn is_reliable(&self) -> bool {
        true
    }

    pub fn is_streamed(&self) -> bool {
        true
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.tx.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.tx.write_all(buffer).await
    }
}

impl<T: AbstractedTcpRx> LinkTcpRx<T> {
    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn is_reliable(&self) -> bool {
        true
    }

    pub fn is_streamed(&self) -> bool {
        true
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.rx.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.rx.read_exact(buffer).await
    }
}
