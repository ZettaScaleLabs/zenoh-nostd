use core::fmt::Display;

use zenoh_proto::{
    TransportError,
    msgs::{NetworkMessage, NetworkMessageRef, TransportMessage},
};

pub trait ZTransportRx {
    fn decode(&mut self, read: &[u8]) -> core::result::Result<(), TransportError>;
    fn decode_with<E>(
        &mut self,
        read: impl FnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> core::result::Result<(), TransportError>
    where
        E: Display;

    fn decode_with_async<E>(
        &mut self,
        read: impl AsyncFnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> impl core::future::Future<Output = core::result::Result<(), TransportError>>
    where
        E: Display;

    fn flush(&mut self) -> impl Iterator<Item = NetworkMessage<'_>>;
}

pub trait ZTransportTx {
    fn encode_t<'a>(&mut self, msg: impl Iterator<Item = TransportMessage<'a>>);
    fn encode<'a>(&mut self, msgs: impl Iterator<Item = NetworkMessage<'a>>);
    fn encode_ref<'a>(&mut self, msgs: impl Iterator<Item = NetworkMessageRef<'a>>);
    fn flush(&mut self) -> Option<&'_ [u8]>;
}
