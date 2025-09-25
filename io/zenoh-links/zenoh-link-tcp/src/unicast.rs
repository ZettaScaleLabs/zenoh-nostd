use core::{fmt, net::SocketAddr};

use heapless::{format, String};
use zenoh_platform::tcp::PlatformTcpStream;
use zenoh_protocol::{core::Locator, transport::BatchSize};
use zenoh_result::ZResult;

pub struct LinkUnicastTcp<const S: usize, const D: usize> {
    // The underlying socket as returned from the tokio library
    socket: PlatformTcpStream,
    // The source socket address of this link (address used on the local host)
    src_addr: SocketAddr,
    src_locator: Locator<S>,
    // The destination socket address of this link (address used on the remote host)
    dst_addr: SocketAddr,
    dst_locator: Locator<D>,
}

impl<const S: usize, const D: usize> fmt::Display for LinkUnicastTcp<S, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.src_addr, self.dst_addr)?;
        Ok(())
    }
}

impl<const S: usize, const D: usize> fmt::Debug for LinkUnicastTcp<S, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tcp")
            .field("src", &self.src_addr)
            .field("dst", &self.dst_addr)
            .field("mtu", &self.get_mtu())
            .finish()
    }
}

impl<const S: usize, const D: usize> LinkUnicastTcp<S, D> {
    pub fn new(
        socket: PlatformTcpStream,
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
    ) -> LinkUnicastTcp<S, D> {
        let src_address: String<S> = format!("{}", src_addr).unwrap();
        let dst_address: String<D> = format!("{}", dst_addr).unwrap();

        LinkUnicastTcp {
            socket,
            src_addr,
            src_locator: Locator::new("tcp", src_address, "").unwrap(),
            dst_addr,
            dst_locator: Locator::new("tcp", dst_address, "").unwrap(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        self.socket.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.socket.write_all(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        self.socket.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        self.socket.read_exact(buffer).await
    }

    #[inline(always)]
    pub fn get_src(&self) -> &Locator<S> {
        &self.src_locator
    }

    #[inline(always)]
    pub fn get_dst(&self) -> &Locator<D> {
        &self.dst_locator
    }

    #[inline(always)]
    pub fn get_mtu(&self) -> BatchSize {
        self.socket.mtu()
    }

    #[inline(always)]
    pub fn is_reliable(&self) -> bool {
        true
    }

    #[inline(always)]
    pub fn is_streamed(&self) -> bool {
        true
    }
}
