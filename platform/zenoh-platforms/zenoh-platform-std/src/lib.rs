use core::future::Future;

use zenoh_platform::{ws::DummyPlatformWSStream, Platform};
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

pub mod tcp;

pub struct PlatformStd;

impl Platform for PlatformStd {
    type PlatformTcpStream = tcp::PlatformStdTcpStream;
    type PlatformWSStream = DummyPlatformWSStream;

    fn new_tcp_stream(
        &mut self,
        addr: &core::net::SocketAddr,
    ) -> impl Future<Output = ZResult<Self::PlatformTcpStream>> {
        async move {
            let socket = async_net::TcpStream::connect(addr)
                .await
                .map_err(|_| zerr!(ZE::ConnectionRefused))
                .context(zctx!("creating async_net Tcp Stream"))?;

            if let Err(err) = socket.set_nodelay(true) {
                log::info!(
                    "Unable to set NODELAY option on TCP link {:?} => {:?}: {}",
                    socket.local_addr(),
                    addr,
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

            Ok(Self::PlatformTcpStream { socket, mtu })
        }
    }
}
