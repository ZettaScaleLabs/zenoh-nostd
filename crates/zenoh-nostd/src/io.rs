#![cfg_attr(not(feature = "std"), no_std)]

mod link;
mod transport;

use core::net::SocketAddr;

pub use link::*;
pub use transport::*;

pub trait ZLinkManager: Sized {
    type Tcp<'p>: ZLink;
    type Udp<'p>: ZLink;
    type Ws<'p>: ZLink;
    type Serial<'p>: ZLink;

    fn connect_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::ConnectionError>>
    {
        async move { unimplemented!("{addr}") }
    }

    fn listen_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::ConnectionError>>
    {
        async move { unimplemented!("{addr}") }
    }

    fn connect_udp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::ConnectionError>>
    {
        async move { unimplemented!("{addr}") }
    }

    fn listen_udp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::ConnectionError>>
    {
        async move { unimplemented!("{addr}") }
    }
}
