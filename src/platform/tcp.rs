use crate::{platform::ZCommunicationError, result::ZResult};

pub trait AbstractedTcpStream {
    type Tx<'a>: AbstractedTcpTx
    where
        Self: 'a;

    type Rx<'a>: AbstractedTcpRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);

    fn mtu(&self) -> u16;

    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>>;

    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>>;
}

pub trait AbstractedTcpTx {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>>;
}

pub trait AbstractedTcpRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>>;
}

pub struct DummyTcpStream;
pub struct DummyTcpTx;
pub struct DummyTcpRx;

impl AbstractedTcpStream for DummyTcpStream {
    type Tx<'a> = DummyTcpTx;
    type Rx<'a> = DummyTcpRx;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        (DummyTcpTx, DummyTcpRx)
    }

    fn mtu(&self) -> u16 {
        0
    }

    async fn write(&mut self, _buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        Err(ZCommunicationError::DidNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        Err(ZCommunicationError::DidNotWrite)
    }

    async fn read(&mut self, _buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        Err(ZCommunicationError::DidNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        Err(ZCommunicationError::DidNotRead)
    }
}

impl AbstractedTcpTx for DummyTcpTx {
    async fn write(&mut self, _buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        Err(ZCommunicationError::DidNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        Err(ZCommunicationError::DidNotWrite)
    }
}

impl AbstractedTcpRx for DummyTcpRx {
    async fn read(&mut self, _buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        Err(ZCommunicationError::DidNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        Err(ZCommunicationError::DidNotRead)
    }
}
