use crate::{platform::ZCommunicationError, result::ZResult};

pub trait PALWebSocket {
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

pub struct DummyWebSocket;

impl PALWebSocket for DummyWebSocket {
    fn mtu(&self) -> u16 {
        1500
    }

    async fn write(
        &mut self,
        _buffer: &[u8],
    ) -> ZResult<usize, ZCommunicationError> { Err(ZCommunicationError::Invalid) }

    async fn write_all(
        &mut self,
        _buffer: &[u8],
    ) -> ZResult<(), ZCommunicationError> { Err(ZCommunicationError::Invalid) }

    async fn read(
        &mut self,
        _buffer: &mut [u8],
    ) -> ZResult<usize, ZCommunicationError> { Err(ZCommunicationError::Invalid) }

    async fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> ZResult<(), ZCommunicationError> { Err(ZCommunicationError::Invalid) }
}
