use core::net::SocketAddr;

use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Read, Write};
use zenoh_platform::tcp::PlatformTcpStream;
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

pub struct PlatformEmbassyTcpStream {
    pub socket: TcpSocket<'static>,

    pub local_addr: SocketAddr,
    pub peer_addr: SocketAddr,
}

impl PlatformTcpStream for PlatformEmbassyTcpStream {
    fn mtu(&self) -> u16 {
        1024
    }

    fn local_addr(&self) -> ZResult<SocketAddr> {
        Ok(self.local_addr)
    }

    fn peer_addr(&self) -> ZResult<SocketAddr> {
        Ok(self.peer_addr)
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
