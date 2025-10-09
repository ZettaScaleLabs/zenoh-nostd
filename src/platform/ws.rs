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

    fn write(
        &mut self,
        _buffer: &[u8],
    ) -> impl Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::Invalid) }
    }

    fn write_all(
        &mut self,
        _buffer: &[u8],
    ) -> impl Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::Invalid) }
    }

    fn read(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl Future<Output = ZResult<usize, ZCommunicationError>> {
        async { Err(ZCommunicationError::Invalid) }
    }

    fn read_exact(
        &mut self,
        _buffer: &mut [u8],
    ) -> impl Future<Output = ZResult<(), ZCommunicationError>> {
        async { Err(ZCommunicationError::Invalid) }
    }
}
