use core::net::SocketAddr;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use zenoh_platform_std::tcp::PlatformStdTcpStream;
use zenoh_result::ZResult;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub struct PlatformTcpStream(PlatformStdTcpStream);

#[cfg(target_arch = "wasm32")]
pub struct PlatformTcpStream(dummy::PlatformDummyTcpStream);

#[cfg(target_arch = "xtensa")]
pub struct PlatformTcpStream(dummy::PlatformDummyTcpStream);

impl PlatformTcpStream {
    pub async fn new(addr: &SocketAddr) -> ZResult<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            Ok(Self(PlatformStdTcpStream::new(addr).await?))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self(dummy::PlatformDummyTcpStream::new(addr).await?))
        }
        #[cfg(target_arch = "xtensa")]
        {
            Ok(Self(dummy::PlatformDummyTcpStream::new(addr).await?))
        }
    }

    pub fn mtu(&self) -> u16 {
        self.0.mtu()
    }

    pub fn local_addr(&self) -> ZResult<SocketAddr> {
        self.0.local_addr()
    }

    pub fn peer_addr(&self) -> ZResult<SocketAddr> {
        self.0.peer_addr()
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        self.0.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.0.write_all(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        self.0.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        self.0.read_exact(buffer).await
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod dummy {
    use core::net::SocketAddr;
    use zenoh_result::{zerr, ZResult, ZE};

    pub struct PlatformDummyTcpStream;

    impl PlatformDummyTcpStream {
        pub async fn new(_addr: &SocketAddr) -> ZResult<Self> {
            Ok(Self)
        }

        pub fn mtu(&self) -> u16 {
            0
        }

        pub fn local_addr(&self) -> ZResult<SocketAddr> {
            Err(zerr!(ZE::UnsupportedPlatform))
        }

        pub fn peer_addr(&self) -> ZResult<SocketAddr> {
            Err(zerr!(ZE::UnsupportedPlatform))
        }

        pub async fn write(&mut self, _buffer: &[u8]) -> ZResult<usize> {
            Ok(0)
        }

        pub async fn write_all(&mut self, _buffer: &[u8]) -> ZResult<()> {
            Ok(())
        }

        pub async fn read(&mut self, _buffer: &mut [u8]) -> ZResult<usize> {
            Ok(0)
        }

        pub async fn read_exact(&mut self, _buffer: &mut [u8]) -> ZResult<()> {
            Ok(())
        }
    }
}
