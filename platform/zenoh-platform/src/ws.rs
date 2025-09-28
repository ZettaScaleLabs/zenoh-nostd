use core::{future::Future, net::SocketAddr};

use zenoh_result::ZResult;

pub trait PlatformWSStream {
    fn connect(addr: &SocketAddr) -> impl Future<Output = ZResult<Self>>
    where
        Self: Sized;

    fn mtu(&self) -> u16;

    fn local_addr(&self) -> ZResult<SocketAddr>;

    fn peer_addr(&self) -> ZResult<SocketAddr>;

    fn write(&mut self, buffer: &[u8]) -> impl Future<Output = ZResult<usize>>;

    fn write_all(&mut self, buffer: &[u8]) -> impl Future<Output = ZResult<()>>;

    fn read(&mut self, buffer: &mut [u8]) -> impl Future<Output = ZResult<usize>>;

    fn read_exact(&mut self, buffer: &mut [u8]) -> impl Future<Output = ZResult<()>>;
}

pub struct DummyPlatformWSStream;

impl PlatformWSStream for DummyPlatformWSStream {
    fn connect(_addr: &SocketAddr) -> impl Future<Output = ZResult<Self>> {
        async { Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform)) }
    }

    fn mtu(&self) -> u16 {
        0
    }

    fn local_addr(&self) -> ZResult<SocketAddr> {
        Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform))
    }

    fn peer_addr(&self) -> ZResult<SocketAddr> {
        Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform))
    }

    fn write(&mut self, _buffer: &[u8]) -> impl Future<Output = ZResult<usize>> {
        async { Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform)) }
    }

    fn write_all(&mut self, _buffer: &[u8]) -> impl Future<Output = ZResult<()>> {
        async { Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform)) }
    }

    fn read(&mut self, _buffer: &mut [u8]) -> impl Future<Output = ZResult<usize>> {
        async { Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform)) }
    }

    fn read_exact(&mut self, _buffer: &mut [u8]) -> impl Future<Output = ZResult<()>> {
        async { Err(zenoh_result::zerr!(zenoh_result::ZE::UnsupportedPlatform)) }
    }
}
