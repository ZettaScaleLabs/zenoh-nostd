use core::net::SocketAddr;

use crate::{
    platform::{ZCommunicationError, ws::PALWebSocket},
    result::ZResult,
};

pub struct LinkWs<T: PALWebSocket> {
    socket: T,
    dst_addr: SocketAddr,
}

impl<T: PALWebSocket> LinkWs<T> {
    pub fn new(socket: T, dst_addr: SocketAddr) -> Self {
        Self { socket, dst_addr }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.socket.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.socket.write_all(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.socket.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.socket.read_exact(buffer).await
    }

    pub fn get_dst(&self) -> &SocketAddr {
        &self.dst_addr
    }

    pub fn get_mtu(&self) -> u16 {
        self.socket.mtu()
    }

    pub fn is_reliable(&self) -> bool {
        true
    }

    pub fn is_streamed(&self) -> bool {
        false
    }
}
