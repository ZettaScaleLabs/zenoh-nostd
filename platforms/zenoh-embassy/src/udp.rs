use core::cell::RefCell;

use embassy_net::udp::{UdpMetadata, UdpSocket};
use zenoh_io::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};

use crate::BufferPoolDrop;

pub struct EmbassyUdpLink<'a> {
    socket: UdpSocket<'a>,
    addr: UdpMetadata,
    mtu: u16,

    idx1: usize,
    pool1: &'a RefCell<dyn BufferPoolDrop>,

    idx2: usize,
    pool2: &'a RefCell<dyn BufferPoolDrop>,
}

impl<'a> EmbassyUdpLink<'a> {
    pub fn new(
        socket: UdpSocket<'a>,
        metadata: UdpMetadata,
        mtu: u16,
        idx1: usize,
        pool1: &'a RefCell<dyn BufferPoolDrop>,
        idx2: usize,
        pool2: &'a RefCell<dyn BufferPoolDrop>,
    ) -> Self {
        Self {
            socket,
            addr: metadata,
            mtu,
            idx1,
            pool1,
            idx2,
            pool2,
        }
    }
}

impl Drop for EmbassyUdpLink<'_> {
    fn drop(&mut self) {
        self.pool1.borrow_mut().release(self.idx1);
        self.pool2.borrow_mut().release(self.idx2);
    }
}

pub struct EmbassyUdpLinkTx<'a> {
    socket: &'a UdpSocket<'a>,
    mtu: u16,
    addr: UdpMetadata,
}

pub struct EmbassyUdpLinkRx<'a> {
    socket: &'a UdpSocket<'a>,
    mtu: u16,
}

impl<'a> ZLinkInfo for EmbassyUdpLink<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        false
    }
}

impl<'a> ZLinkInfo for EmbassyUdpLinkTx<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        false
    }
}

impl<'a> ZLinkInfo for EmbassyUdpLinkRx<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        false
    }
}

impl<'a> ZLinkTx for EmbassyUdpLink<'a> {
    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .send_to(buffer, self.addr)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotWrite)
    }
}

impl<'a> ZLinkTx for EmbassyUdpLinkTx<'a> {
    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .send_to(buffer, self.addr)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotWrite)
    }
}

impl<'a> ZLinkRx for EmbassyUdpLink<'a> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_proto::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
            .map(|m| m.0)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
            .map(|_| ())
    }
}

impl<'a> ZLinkRx for EmbassyUdpLinkRx<'a> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_proto::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
            .map(|m| m.0)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .recv_from(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
            .map(|_| ())
    }
}

impl<'a> ZLink for EmbassyUdpLink<'a> {
    type Tx<'b>
        = EmbassyUdpLinkTx<'b>
    where
        Self: 'b;

    type Rx<'b>
        = EmbassyUdpLinkRx<'b>
    where
        Self: 'b;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = EmbassyUdpLinkTx {
            socket: &self.socket,
            mtu: self.mtu,
            addr: self.addr,
        };
        let rx = EmbassyUdpLinkRx {
            socket: &self.socket,
            mtu: self.mtu,
        };
        (tx, rx)
    }
}
