#![no_std]

use ::core::mem;

use zenoh_result::{zbail, ZResult, ZE};

pub mod common;
pub mod core;

pub(crate) trait Writer {
    fn write_u8(&mut self, byte: u8) -> ZResult<usize>;
    fn write(&mut self, buffer: &[u8], len: usize) -> ZResult<usize>;
}

impl Writer for &mut [u8] {
    fn write_u8(&mut self, byte: u8) -> ZResult<usize> {
        self.write(&[byte], 1)
    }

    fn write(&mut self, buffer: &[u8], len: usize) -> ZResult<usize> {
        let len = len.min(self.len());
        if buffer.len() < len {
            zbail!(ZE::CapacityExceeded)
        }

        if len == 0 {
            return Ok(0);
        }

        let (to_write, remaining) = mem::take(self).split_at_mut(len);
        to_write.copy_from_slice(&buffer[..len]);

        *self = remaining;

        Ok(len)
    }
}

pub trait WCodec<Message> {
    fn write(&self, message: &Message, support: &mut [u8]) -> ZResult<usize>;
}

pub trait RCodec<Message> {
    fn read(&self, support: &[u8]) -> ZResult<(Message, usize)>;
}

pub struct Zenoh080;
