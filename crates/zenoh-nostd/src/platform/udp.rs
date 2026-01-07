pub trait ZUdpSocket: ZUdpTx + ZUdpRx {
    type Tx<'a>: ZUdpTx
    where
        Self: 'a;

    type Rx<'a>: ZUdpRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);

    fn mtu(&self) -> u16;
}

pub trait ZUdpTx {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub trait ZUdpRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub struct DummyUdpSocket;
pub struct DummyUdpTx;
pub struct DummyUdpRx;

impl ZUdpSocket for DummyUdpSocket {
    type Tx<'a> = DummyUdpTx;
    type Rx<'a> = DummyUdpRx;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        (DummyUdpTx, DummyUdpRx)
    }

    fn mtu(&self) -> u16 {
        0
    }
}

impl ZUdpTx for DummyUdpSocket {
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZUdpTx for DummyUdpTx {
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZUdpRx for DummyUdpSocket {
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

impl ZUdpRx for DummyUdpRx {
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
