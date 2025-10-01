#![no_std]

use zenoh_buffer::{ZBufReader, ZBufWriter};
use zenoh_result::ZResult;

pub mod common;
pub mod core;

pub trait WCodec<'a, Message> {
    fn write(&self, message: Message, writer: &mut ZBufWriter<'a>) -> ZResult<()>;
}

pub trait RCodec<'a, Message> {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<Message>;
}

pub struct Zenoh080;
