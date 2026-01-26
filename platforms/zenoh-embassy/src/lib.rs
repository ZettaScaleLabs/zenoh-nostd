#![no_std]

use core::cell::RefCell;
use embassy_net::{
    IpAddress, IpEndpoint, Stack,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use zenoh_nostd::platform::*;

pub mod tcp;
pub mod udp;

struct BufferPool<T, const MTU: usize, const SOCKS: usize> {
    tx_buffers: [[T; MTU]; SOCKS],
    rx_buffers: [[T; MTU]; SOCKS],
    used: [bool; SOCKS],
}

impl<T, const MTU: usize, const SOCKS: usize> BufferPool<T, MTU, SOCKS> {
    fn new(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            tx_buffers: [[value; MTU]; SOCKS],
            rx_buffers: [[value; MTU]; SOCKS],
            used: [false; SOCKS],
        }
    }

    fn allocate(&mut self) -> Option<usize> {
        self.used.iter().position(|&used| !used).map(|idx| {
            self.used[idx] = true;
            idx
        })
    }
}

pub(crate) trait BufferPoolDrop {
    fn release(&mut self, idx: usize);
}

impl<T, const MTU: usize, const SOCKS: usize> BufferPoolDrop for BufferPool<T, MTU, SOCKS> {
    fn release(&mut self, idx: usize) {
        if idx < SOCKS {
            self.used[idx] = false;
        }
    }
}

pub struct EmbassyLinkManager<'a, const MTU: usize, const SOCKS: usize> {
    stack: Stack<'a>,
    buffers: RefCell<BufferPool<u8, MTU, SOCKS>>,
    metadatas: RefCell<BufferPool<PacketMetadata, MTU, SOCKS>>,
}

impl<'a, const MTU: usize, const SOCKS: usize> EmbassyLinkManager<'a, MTU, SOCKS> {
    pub fn new(stack: Stack<'a>) -> Self {
        Self {
            stack,
            buffers: RefCell::new(BufferPool::new(0)),
            metadatas: RefCell::new(BufferPool::new(PacketMetadata::EMPTY)),
        }
    }

    fn allocate_buffers(&self) -> Option<(usize, &mut [u8], &mut [u8])> {
        let idx = self.buffers.borrow_mut().allocate()?;

        // SAFETY: This pool is simple, I should not have made any mistake. The reference will still be valid
        // because Tcp borrows EmbassyLinkManager and so EmbassyLinkManager can't be moved.
        let buffers = unsafe { &mut *self.buffers.as_ptr() };
        let tx = &mut buffers.tx_buffers[idx];
        let rx = &mut buffers.rx_buffers[idx];

        Some((idx, tx, rx))
    }

    fn allocate_metadatas(&self) -> Option<(usize, &mut [PacketMetadata], &mut [PacketMetadata])> {
        let idx = self.metadatas.borrow_mut().allocate()?;

        // SAFETY: This pool is simple, I should not have made any mistake. The reference will still be valid
        // because Tcp borrows EmbassyLinkManager and so EmbassyLinkManager can't be moved.
        let buffers = unsafe { &mut *self.metadatas.as_ptr() };
        let tx = &mut buffers.tx_buffers[idx];
        let rx = &mut buffers.rx_buffers[idx];

        Some((idx, tx, rx))
    }
}

impl<'a, const MTU: usize, const SOCKS: usize> ZLinkManager for EmbassyLinkManager<'a, MTU, SOCKS> {
    type Tcp<'p> = tcp::EmbassyTcpLink<'p>;
    type Udp<'p> = udp::EmbassyUdpLink<'p>;
    type Serial<'p> = ();
    type Ws<'p> = ();

    async fn connect_tcp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let (idx, tx, rx) = self.allocate_buffers().ok_or(LinkError::CouldNotConnect)?;
        let mut socket = TcpSocket::new(self.stack.clone(), rx, tx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => {
                zbail!(LinkError::CouldNotConnect)
            }
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket.connect(ip_endpoint).await.map_err(|e| {
            error!("Could not connect to {:?}: {:?}", addr, e);
            LinkError::CouldNotConnect
        })?;

        Ok(Link::Tcp(Self::Tcp::new(
            socket,
            MTU as u16,
            idx,
            &self.buffers,
        )))
    }

    async fn listen_tcp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let (idx, tx, rx) = self.allocate_buffers().ok_or(LinkError::CouldNotConnect)?;
        let mut socket = TcpSocket::new(self.stack.clone(), rx, tx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => {
                zbail!(LinkError::CouldNotConnect)
            }
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket.accept(ip_endpoint).await.map_err(|e| {
            error!("Could not connect to {:?}: {:?}", addr, e);
            LinkError::CouldNotConnect
        })?;

        Ok(Link::Tcp(Self::Tcp::new(
            socket,
            MTU as u16,
            idx,
            &self.buffers,
        )))
    }

    async fn connect_udp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let (idx1, tx, rx) = self.allocate_buffers().ok_or(LinkError::CouldNotConnect)?;

        let (idx2, tm, rm) = self
            .allocate_metadatas()
            .ok_or(LinkError::CouldNotConnect)?;

        let mut socket = UdpSocket::new(self.stack, rm, rx, tm, tx);
        socket.bind(0).map_err(|_| LinkError::CouldNotConnect)?;

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => {
                zbail!(LinkError::CouldNotConnect)
            }
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        Ok(Link::Udp(Self::Udp::new(
            socket,
            ip_endpoint.into(),
            MTU as u16,
            idx1,
            &self.buffers,
            idx2,
            &self.metadatas,
        )))
    }

    async fn listen_udp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let (idx1, tx, rx) = self.allocate_buffers().ok_or(LinkError::CouldNotConnect)?;

        let (idx2, tm, rm) = self
            .allocate_metadatas()
            .ok_or(LinkError::CouldNotConnect)?;

        let mut socket = UdpSocket::new(self.stack, rm, rx, tm, tx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => {
                zbail!(LinkError::CouldNotConnect)
            }
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());
        socket
            .bind(ip_endpoint)
            .map_err(|_| LinkError::CouldNotConnect)?;

        Ok(Link::Udp(Self::Udp::new(
            socket,
            ip_endpoint.into(),
            MTU as u16,
            idx1,
            &self.buffers,
            idx2,
            &self.metadatas,
        )))
    }
}
