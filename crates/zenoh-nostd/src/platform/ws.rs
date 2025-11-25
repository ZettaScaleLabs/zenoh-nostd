use zenoh_proto::ZResult;

use crate::platform::ZConnectionError;

pub trait AbstractedWsStream {
    type Tx<'a>: AbstractedWsTx
    where
        Self: 'a;

    type Rx<'a>: AbstractedWsRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);

    fn mtu(&self) -> u16;

    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZConnectionError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZConnectionError>>;

    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZConnectionError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZConnectionError>>;
}

pub trait AbstractedWsTx {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZConnectionError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZConnectionError>>;
}

pub trait AbstractedWsRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<usize, ZConnectionError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = ZResult<(), ZConnectionError>>;
}

pub struct DummyWsStream;
pub struct DummyWsTx;
pub struct DummyWsRx;

impl AbstractedWsStream for DummyWsStream {
    type Tx<'a> = DummyWsTx;
    type Rx<'a> = DummyWsRx;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        (DummyWsTx, DummyWsRx)
    }

    fn mtu(&self) -> u16 {
        0
    }

    async fn write(&mut self, _buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        Err(ZConnectionError::CouldNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        Err(ZConnectionError::CouldNotWrite)
    }

    async fn read(&mut self, _buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        Err(ZConnectionError::CouldNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        Err(ZConnectionError::CouldNotRead)
    }
}

impl AbstractedWsTx for DummyWsTx {
    async fn write(&mut self, _buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        Err(ZConnectionError::CouldNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        Err(ZConnectionError::CouldNotWrite)
    }
}

impl AbstractedWsRx for DummyWsRx {
    async fn read(&mut self, _buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        Err(ZConnectionError::CouldNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        Err(ZConnectionError::CouldNotRead)
    }
}
