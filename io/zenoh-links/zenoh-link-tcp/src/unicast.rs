use core::{fmt, net::SocketAddr};

use async_net::TcpStream;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use zenoh_link_commons::LinkAuthId;
use zenoh_protocol::{core::Locator, transport::BatchSize};
use zenoh_result::{zerror, ZResult};

pub struct LinkUnicastTcp {
    // The underlying socket as returned from the tokio library
    socket: TcpStream,
    // The source socket address of this link (address used on the local host)
    src_addr: SocketAddr,
    src_locator: Locator,
    // The destination socket address of this link (address used on the remote host)
    dst_addr: SocketAddr,
    dst_locator: Locator,
    // The computed mtu
    mtu: BatchSize,
}

impl fmt::Display for LinkUnicastTcp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.src_addr, self.dst_addr)?;
        Ok(())
    }
}

impl fmt::Debug for LinkUnicastTcp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tcp")
            .field("src", &self.src_addr)
            .field("dst", &self.dst_addr)
            .field("mtu", &self.get_mtu())
            .finish()
    }
}
impl LinkUnicastTcp {
    pub fn new(socket: TcpStream, src_addr: SocketAddr, dst_addr: SocketAddr) -> LinkUnicastTcp {
        if let Err(err) = socket.set_nodelay(true) {
            println!(
                "Unable to set NODELAY option on TCP link {:?} => {:?}: {}",
                src_addr, dst_addr, err
            );
        }

        // Compute the MTU
        // See IETF RFC6691: https://datatracker.ietf.org/doc/rfc6691/
        let header = match src_addr.ip() {
            core::net::IpAddr::V4(_) => 40,
            core::net::IpAddr::V6(_) => 60,
        };
        #[allow(unused_mut)] // mut is not needed when target_family != unix
        let mut mtu = BatchSize::MAX - header;

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
            mtu = (mtu as u32).min(tgt) as BatchSize;
        }

        // Build the Tcp object
        LinkUnicastTcp {
            socket,
            src_addr,
            src_locator: Locator::new("tcp", src_addr.to_string(), "").unwrap(),
            dst_addr,
            dst_locator: Locator::new("tcp", dst_addr.to_string(), "").unwrap(),
            mtu,
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        self.socket.write(buffer).await.map_err(|e| {
            let e = zerror!("Write error on TCP link {}: {}", self, e);
            e.into()
        })
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.socket.write_all(buffer).await.map_err(|e| {
            let e = zerror!("Write error on TCP link {}: {}", self, e);
            e.into()
        })
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        self.socket.read(buffer).await.map_err(|e| {
            let e = zerror!("Read error on TCP link {}: {}", self, e);
            e.into()
        })
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        self.socket.read_exact(buffer).await.map_err(|e| {
            let e = zerror!("Read error on TCP link {}: {}", self, e);
            e.into()
        })
    }

    #[inline(always)]
    pub fn get_src(&self) -> &Locator {
        &self.src_locator
    }

    #[inline(always)]
    pub fn get_dst(&self) -> &Locator {
        &self.dst_locator
    }

    #[inline(always)]
    pub fn get_mtu(&self) -> BatchSize {
        self.mtu
    }

    #[inline(always)]
    pub fn is_reliable(&self) -> bool {
        true
    }

    #[inline(always)]
    pub fn is_streamed(&self) -> bool {
        true
    }

    #[inline(always)]
    pub fn get_auth_id(&self) -> &LinkAuthId {
        &LinkAuthId::Tcp
    }
}
