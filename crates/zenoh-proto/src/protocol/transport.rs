use crate::{
    ZCodecResult, ZExt, ZReader, ZReaderExt,
    network::NetworkBatch,
    transport::{
        close::Close,
        frame::{Frame, FrameHeader},
        init::{InitAck, InitSyn},
        keepalive::KeepAlive,
        open::{OpenAck, OpenSyn},
    },
};

#[cfg(test)]
use {
    crate::{ZWriter, ZWriterExt},
    rand::{Rng, thread_rng},
};

pub mod close;
pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

mod batch;
pub use batch::*;

#[derive(Debug, PartialEq)]
pub enum TransportBody<'a, 'b> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    Close(Close),
    KeepAlive(KeepAlive),
    Frame(Frame<'a, 'b>),
}

pub struct TransportBatch<'a> {
    reader: ZReader<'a>,
}

impl<'a> TransportBatch<'a> {
    pub fn new(reader: ZReader<'a>) -> TransportBatch<'a> {
        TransportBatch { reader }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ZCodecResult<TransportBody<'a, '_>>> {
        if !self.reader.can_read() {
            return None;
        }

        let mark = self.reader.mark();
        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {
                match <$ty as $crate::ZBodyDecode>::z_body_decode(&mut self.reader, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            };
        }

        let ack = header & 0b0010_0000 != 0;
        let body = match header & 0b0001_1111 {
            InitAck::ID if ack => TransportBody::InitAck(decode!(InitAck)),
            InitSyn::ID => TransportBody::InitSyn(decode!(InitSyn)),
            OpenAck::ID if ack => TransportBody::OpenAck(decode!(OpenAck)),
            OpenSyn::ID => TransportBody::OpenSyn(decode!(OpenSyn)),
            Close::ID => TransportBody::Close(decode!(Close)),
            KeepAlive::ID => TransportBody::KeepAlive(decode!(KeepAlive)),
            Frame::ID => {
                let frame = decode!(FrameHeader);
                let iter = NetworkBatch::new(&mut self.reader);
                TransportBody::Frame(Frame {
                    header: frame,
                    msgs: iter,
                })
            }
            _ => {
                self.reader.rewind(mark);
                return None;
            }
        };

        Some(Ok(body))
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasQoS {}

#[derive(ZExt, Debug, PartialEq)]
pub struct Auth<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

impl<'a> Auth<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct MultiLink<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

impl<'a> MultiLink<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasLowLatency {}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasCompression {}

#[derive(ZExt, Debug, PartialEq)]
pub struct Patch {
    pub int: u8,
}

impl Patch {
    pub const NONE: Self = Self { int: 0 };
    pub const CURRENT: Self = Self { int: 1 };

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        Self {
            int: thread_rng().r#gen(),
        }
    }
}
