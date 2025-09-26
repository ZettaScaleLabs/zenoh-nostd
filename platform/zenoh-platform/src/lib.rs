#![no_std]

use core::future::Future;

use zenoh_result::{zerr, ZResult, ZE};

pub mod tcp;
pub mod ws;

pub trait Platform {
    type PlatformTcpStream: tcp::PlatformTcpStream;
    type PlatformWSStream: ws::PlatformWSStream;

    fn new_tcp_stream(
        &mut self,
        addr: &core::net::SocketAddr,
    ) -> impl Future<Output = ZResult<Self::PlatformTcpStream>> {
        let _addr = addr;

        async { Err(zerr!(ZE::UnsupportedPlatform)) }
    }

    fn new_ws_stream(
        &mut self,
        addr: &core::net::SocketAddr,
    ) -> impl Future<Output = ZResult<Self::PlatformWSStream>> {
        let _addr = addr;

        async { Err(zerr!(ZE::UnsupportedPlatform)) }
    }
}
