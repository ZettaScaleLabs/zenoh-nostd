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

pub struct TransportBatch<'a, 'b> {
    reader: &'b mut ZReader<'a>,
}

impl<'a, 'b> TransportBatch<'a, 'b> {
    pub fn new(reader: &'b mut ZReader<'a>) -> TransportBatch<'a, 'b> {
        TransportBatch { reader }
    }

    pub fn mark(&self) -> &'a [u8] {
        self.reader.mark()
    }

    pub fn rewind(&mut self, mark: &'a [u8]) {
        self.reader.rewind(mark);
    }

    pub fn next_mark(&mut self) -> ZCodecResult<Option<(&'a [u8], TransportBody<'a, '_>)>> {
        let mark = self.reader.mark();
        match self.next()? {
            Some(body) => Ok(Some((mark, body))),
            None => Ok(None),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> ZCodecResult<Option<TransportBody<'a, '_>>> {
        if !self.reader.can_read() {
            return Ok(None);
        }

        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {
                <$ty as $crate::ZBodyDecode>::z_body_decode(self.reader, header)?
            };
        }

        let ack = header & 0b0010_0000 != 0;
        Ok(Some(match header & 0b0001_1111 {
            InitAck::ID if ack => TransportBody::InitAck(decode!(InitAck)),
            InitSyn::ID => TransportBody::InitSyn(decode!(InitSyn)),
            OpenAck::ID if ack => TransportBody::OpenAck(decode!(OpenAck)),
            OpenSyn::ID => TransportBody::OpenSyn(decode!(OpenSyn)),
            Close::ID => TransportBody::Close(decode!(Close)),
            KeepAlive::ID => TransportBody::KeepAlive(decode!(KeepAlive)),
            Frame::ID => {
                let frame = decode!(FrameHeader);
                let iter = NetworkBatch::new(self.reader);
                TransportBody::Frame(Frame {
                    header: frame,
                    msgs: iter,
                })
            }
            _ => {
                return Err(crate::ZCodecError::CouldNotRead);
            }
        }))
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
