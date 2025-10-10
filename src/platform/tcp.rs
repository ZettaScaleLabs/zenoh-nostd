use crate::{platform::ZCommunicationError, result::ZResult};

pub trait AbstractedTcpStream {
    type Tx: AbstractedTcpTx;
    type Rx: AbstractedTcpRx;

    fn split(self) -> (Self::Tx, Self::Rx);

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
    type Tx = DummyTcpTx;
    type Rx = DummyTcpRx;

    fn split(self) -> (Self::Tx, Self::Rx) {
        (DummyTcpTx, DummyTcpRx)
    }

    fn mtu(&self) -> u16 {
        0
    }

    fn write(
        &mut self,
        _buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotWrite) }
    }

    fn write_all(
        &mut self,
        _buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotWrite) }
    }

    fn read(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotRead) }
    }

    fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotRead) }
    }
}

impl AbstractedTcpTx for DummyTcpTx {
    fn write(
        &mut self,
        _buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotWrite) }
    }

    fn write_all(
        &mut self,
        _buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotWrite) }
    }
}

impl AbstractedTcpRx for DummyTcpRx {
    fn read(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotRead) }
    }

    fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::DidNotRead) }
    }
}
