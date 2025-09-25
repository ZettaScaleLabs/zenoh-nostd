use core::net::SocketAddr;

use futures_lite::{AsyncReadExt, AsyncWriteExt};

use zenoh_result::ZResult;
use zenoh_result::{zerr, ZE};

pub struct PlatformStdTcpStream {
    socket: async_net::TcpStream,
    mtu: u16,
}

impl PlatformStdTcpStream {
    pub async fn new(dst_addr: &SocketAddr) -> ZResult<Self> {
        let socket = async_net::TcpStream::connect(dst_addr)
            .await
            .map_err(|_| zerr!(ZE::ConnectionRefused))?;

        if let Err(err) = socket.set_nodelay(true) {
            println!(
                "Unable to set NODELAY option on TCP link {:?} => {:?}: {}",
                socket.local_addr(),
                dst_addr,
                err
            );
        }

        let header = match socket
            .local_addr()
            .map_err(|_| zerr!(ZE::InvalidAddress))?
            .ip()
        {
            core::net::IpAddr::V4(_) => 40,
            core::net::IpAddr::V6(_) => 60,
        };

        #[allow(unused_mut)] // mut is not needed when target_family != unix
        let mut mtu = u16::MAX - header;

        // target limitation of socket2: https://docs.rs/socket2/latest/src/socket2/sys/unix.rs.html#1544
        #[cfg(target_family = "unix")]
        {
            let socket = socket2::SockRef::from(&socket);
            // Get the MSS and divide it by 2 to ensure we can at least fill half the MSS
            let mss = socket.mss().unwrap_or(mtu as u32) / 2;
            // Compute largest multiple of TCP MSS that is smaller of default MTU
            let mut tgt = mss;
            while (tgt + mss) < mtu as u32 {
                tgt += mss;
            }
            mtu = (mtu as u32).min(tgt) as u16;
        }

        Ok(Self { socket, mtu })
    }

    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn local_addr(&self) -> ZResult<SocketAddr> {
        self.socket
            .local_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))
    }

    pub fn peer_addr(&self) -> ZResult<SocketAddr> {
        self.socket
            .peer_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        self.socket
            .write(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntRead))
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntRead))
    }
}
