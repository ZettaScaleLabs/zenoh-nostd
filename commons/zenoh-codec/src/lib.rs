#![no_std]

use zenoh_buffer::{ZBufReader, ZBufWriter};
use zenoh_protocol::core::Reliability;
use zenoh_result::{zbail, ZResult, ZE};

pub mod common;
pub mod core;
pub mod network;
pub mod transport;
pub mod zenoh;

pub trait LCodec<'a, Message> {
    fn w_len(&self, message: Message) -> usize;
}

pub trait WCodec<'a, Message> {
    fn write(&self, message: Message, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let _ = (message, writer);

        zbail!(ZE::UnImplemented)
    }

    fn write_without_length(&self, message: Message, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let _ = (message, writer);

        zbail!(ZE::UnImplemented)
    }
}

pub trait RCodec<'a, Message> {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<Message> {
        let _ = reader;

        zbail!(ZE::UnImplemented)
    }

    fn read_with_condition(
        &self,
        reader: &mut ZBufReader<'a>,
        condition: bool,
    ) -> ZResult<Message> {
        let _ = (reader, condition);

        zbail!(ZE::UnImplemented)
    }

    fn read_knowing_header(&self, reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Message> {
        let _ = (reader, header);

        zbail!(ZE::UnImplemented)
    }

    fn read_knowing_length(&self, reader: &mut ZBufReader<'a>, length: usize) -> ZResult<Message> {
        let _ = (reader, length);

        zbail!(ZE::UnImplemented)
    }

    fn read_with_reliability(
        &self,
        reader: &mut ZBufReader<'a>,
        reliability: Reliability,
    ) -> ZResult<Message> {
        let _ = (reader, reliability);

        zbail!(ZE::UnImplemented)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZCodec;
