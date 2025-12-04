pub trait ZTcpStream: ZTcpTx + ZTcpRx {
    type Tx<'a>: ZTcpTx
    where
        Self: 'a;

    type Rx<'a>: ZTcpRx
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);

    fn mtu(&self) -> u16;
}

pub trait ZTcpTx {
    fn write(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub trait ZTcpRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<usize, crate::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = core::result::Result<(), crate::LinkError>>;
}

pub struct DummyTcpStream;
pub struct DummyTcpTx;
pub struct DummyTcpRx;

impl ZTcpStream for DummyTcpStream {
    type Tx<'a> = DummyTcpTx;
    type Rx<'a> = DummyTcpRx;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        (DummyTcpTx, DummyTcpRx)
    }

    fn mtu(&self) -> u16 {
        0
    }
}

impl ZTcpTx for DummyTcpStream {
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZTcpTx for DummyTcpTx {
    async fn write(&mut self, _buffer: &[u8]) -> core::result::Result<usize, crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }

    async fn write_all(&mut self, _buffer: &[u8]) -> core::result::Result<(), crate::LinkError> {
        Err(crate::LinkError::LinkTxFailed)
    }
}

impl ZTcpRx for DummyTcpStream {
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

impl ZTcpRx for DummyTcpRx {
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
