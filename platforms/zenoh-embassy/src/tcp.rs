use core::cell::RefCell;

use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embedded_io_async::{Read, Write};
use zenoh_io::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};

use crate::BufferPoolDrop;

pub struct EmbassyTcpLink<'a> {
    socket: TcpSocket<'a>,
    mtu: u16,

    idx: usize,
    pool: &'a RefCell<dyn BufferPoolDrop>,
}

impl<'a> EmbassyTcpLink<'a> {
    pub fn new(
        socket: TcpSocket<'a>,
        mtu: u16,
        idx: usize,
        pool: &'a RefCell<dyn BufferPoolDrop>,
    ) -> Self {
        Self {
            socket,
            mtu,
            idx,
            pool,
        }
    }
}

impl Drop for EmbassyTcpLink<'_> {
    fn drop(&mut self) {
        self.pool.borrow_mut().release(self.idx);
    }
}

pub struct EmbassyTcpLinkTx<'a> {
    socket: TcpWriter<'a>,
    mtu: u16,
}

pub struct EmbassyTcpLinkRx<'a> {
    socket: TcpReader<'a>,
    mtu: u16,
}

impl<'a> ZLinkInfo for EmbassyTcpLink<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        true
    }
}

impl<'a> ZLinkInfo for EmbassyTcpLinkTx<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        true
    }
}

impl<'a> ZLinkInfo for EmbassyTcpLinkRx<'a> {
    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn is_streamed(&self) -> bool {
        true
    }
}

impl<'a> ZLinkTx for EmbassyTcpLink<'a> {
    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotWrite)
    }
}

impl<'a> ZLinkTx for EmbassyTcpLinkTx<'a> {
    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .write_all(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotWrite)
    }
}

impl<'a> ZLinkRx for EmbassyTcpLink<'a> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_proto::LinkError> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
    }
}

impl<'a> ZLinkRx for EmbassyTcpLinkRx<'a> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_proto::LinkError> {
        self.socket
            .read(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_proto::LinkError> {
        self.socket
            .read_exact(buffer)
            .await
            .map_err(|_| zenoh_proto::LinkError::CouldNotRead)
    }
}

impl<'a> ZLink for EmbassyTcpLink<'a> {
    type Tx<'b>
        = EmbassyTcpLinkTx<'b>
    where
        Self: 'b;

    type Rx<'b>
        = EmbassyTcpLinkRx<'b>
    where
        Self: 'b;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let (rx, tx) = self.socket.split();
        let tx = EmbassyTcpLinkTx {
            socket: tx,
            mtu: self.mtu,
        };
        let rx = EmbassyTcpLinkRx {
            socket: rx,
            mtu: self.mtu,
        };
        (tx, rx)
    }
}
