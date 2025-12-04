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
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub trait ZWsRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
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
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZWsTx for DummyWsTx {
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZWsRx for DummyWsStream {
    async fn read(&mut self, _buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkRxFailed)
    }

    async fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkRxFailed)
    }
}
impl ZWsRx for DummyWsRx {
    async fn read(&mut self, _buffer: &mut [u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkRxFailed)
    }

    async fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkRxFailed)
    }
}
