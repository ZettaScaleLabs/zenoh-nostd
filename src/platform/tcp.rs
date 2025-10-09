use crate::{platform::ZCommunicationError, result::ZResult};

pub trait PALTcpStream {
    fn mtu(&self) -> u16;

    fn write(&mut self, buffer: &[u8])
    -> impl Future<Output = ZResult<usize, ZCommunicationError>>;

    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl Future<Output = ZResult<(), ZCommunicationError>>;

    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = ZResult<usize, ZCommunicationError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = ZResult<(), ZCommunicationError>>;
}

pub struct DummyTcpStream;

impl PALTcpStream for DummyTcpStream {
    fn mtu(&self) -> u16 {
        0
    }

    async fn write(
        &mut self,
        _buffer: &[u8],
    ) -> ZResult<usize, ZCommunicationError> { Ok(0) }

    async fn write_all(
        &mut self,
        _buffer: &[u8],
    ) -> ZResult<(), ZCommunicationError> { Ok(()) }

    async fn read(
        &mut self,
        _buffer: &mut [u8],
    ) -> ZResult<usize, ZCommunicationError> { Ok(0) }

    async fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> ZResult<(), ZCommunicationError> { Ok(()) }
}
