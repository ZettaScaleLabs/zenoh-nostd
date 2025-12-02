use crate::ZResult;

pub trait ZWebSocket: ZWsTx + ZWsRx {
    type Tx<'a>: ZWsTx
    where
        Self: 'a;

    type Rx<'a>: ZWsRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);

    fn mtu(&self) -> u16;
}

pub trait ZWsTx {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl ::core::future::Future<Output = ZResult<usize, crate::ZLinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl ::core::future::Future<Output = ZResult<(), crate::ZLinkError>>;
}

pub trait ZWsRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl ::core::future::Future<Output = ZResult<usize, crate::ZLinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl ::core::future::Future<Output = ZResult<(), crate::ZLinkError>>;
}

pub struct DummyWsStream;
pub struct DummyWsTx;
pub struct DummyWsRx;

impl ZWebSocket for DummyWsStream {
    type Tx<'a> = DummyWsTx;
    type Rx<'a> = DummyWsRx;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        (DummyWsTx, DummyWsRx)
    }

    fn mtu(&self) -> u16 {
        0
    }
}

impl ZWsTx for DummyWsStream {
    async fn write(&mut self, _buffer: &[u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> crate::ZResult<(), crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotWrite)
    }
}

impl ZWsTx for DummyWsTx {
    async fn write(&mut self, _buffer: &[u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotWrite)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> crate::ZResult<(), crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotWrite)
    }
}

impl ZWsRx for DummyWsStream {
    async fn read(&mut self, _buffer: &mut [u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> crate::ZResult<(), crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotRead)
    }
}
impl ZWsRx for DummyWsRx {
    async fn read(&mut self, _buffer: &mut [u8]) -> crate::ZResult<usize, crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotRead)
    }

    async fn read_exact(&mut self, _buffer: &mut [u8]) -> crate::ZResult<(), crate::ZLinkError> {
        Err(crate::ZLinkError::CouldNotRead)
    }
}
