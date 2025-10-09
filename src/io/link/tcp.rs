use core::net::SocketAddr;

use crate::{
    platform::{ZCommunicationError, tcp::PALTcpStream},
    result::ZResult,
};

pub struct LinkTcp<T: PALTcpStream> {
    stream: T,
    dst_addr: SocketAddr,
}

impl<T: PALTcpStream> LinkTcp<T> {
    pub fn new(stream: T, dst_addr: SocketAddr) -> Self {
        Self { stream, dst_addr }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream.write(buffer).await
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZCommunicationError> {
        self.stream.write_all(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZCommunicationError> {
        self.stream.read(buffer).await
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZCommunicationError> {
        self.stream.read_exact(buffer).await
    }

    pub fn get_dst(&self) -> &SocketAddr {
        &self.dst_addr
    }

    pub fn get_mtu(&self) -> u16 {
        self.stream.mtu()
    }

    pub fn is_reliable(&self) -> bool {
        true
    }

    pub fn is_streamed(&self) -> bool {
        true
    }
}
