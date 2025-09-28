use core::net::SocketAddr;

use futures_lite::{AsyncReadExt, AsyncWriteExt};

use zenoh_platform::tcp::PlatformTcpStream;
use zenoh_result::{zctx, WithContext, ZResult};
use zenoh_result::{zerr, ZE};

pub struct PlatformStdTcpStream {
    pub socket: async_net::TcpStream,
    pub mtu: u16,
}

impl PlatformTcpStream for PlatformStdTcpStream {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn local_addr(&self) -> ZResult<SocketAddr> {
        self.socket
            .local_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))
            .context(zctx!("getting local addr of tcp stream"))
    }

    fn peer_addr(&self) -> ZResult<SocketAddr> {
        self.socket
            .peer_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))
            .context(zctx!("getting peer addr of tcp stream"))
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        self.socket
            .write(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))
            .context(zctx!("writing to tcp stream"))
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntWrite))
            .context(zctx!("writing all to tcp stream"))
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntRead))
            .context(zctx!("reading from tcp stream"))
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| zerr!(ZE::DidntRead))
            .context(zctx!("reading exact from tcp stream"))
    }
}
