use core::time::Duration;

#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZCodecError, ZCodecResult, ZDecode, ZEncode, ZExt,
    ZExtKind, ZHeader, ZLen, ZReader, ZReaderExt, ZWriter,
    network::{
        declare::Declare,
        interest::Interest,
        push::Push,
        request::Request,
        response::{Response, ResponseFinal},
    },
};

pub mod declare;
pub mod interest;
pub mod push;
pub mod request;
pub mod response;

crate::aggregate_enum_batch! {
    #[derive(Debug, PartialEq)]
    pub enum NetworkBody<'a> {
        Push<'a>,
        Request<'a>,
        Response<'a>,
        ResponseFinal,
        Interest<'a>,
        Declare<'a>,
    }
}

#[derive(Debug, PartialEq)]
pub struct NetworkBatch<'a, 'b> {
    pub reader: &'b mut ZReader<'a>,
}

impl<'a, 'b> NetworkBatch<'a, 'b> {
    pub fn new(reader: &'b mut ZReader<'a>) -> Self {
        Self { reader }
    }
}

impl<'a, 'b> core::iter::Iterator for NetworkBatch<'a, 'b> {
    type Item = ZCodecResult<NetworkBody<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.reader.can_read() {
            return None;
        }

        let mark = self.reader.mark();
        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {{
                match <$ty as ZBodyDecode>::z_body_decode(self.reader, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            }};
        }

        let body = match header & 0b0001_1111 {
            Push::ID => NetworkBody::Push(decode!(Push)),
            Request::ID => NetworkBody::Request(decode!(Request)),
            Response::ID => NetworkBody::Response(decode!(Response)),
            ResponseFinal::ID => NetworkBody::ResponseFinal(decode!(ResponseFinal)),
            Interest::ID => NetworkBody::Interest(decode!(Interest)),
            Declare::ID => NetworkBody::Declare(decode!(Declare)),
            _ => {
                self.reader.rewind(mark);
                return None;
            }
        };

        Some(Ok(body))
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct QoS {
    inner: u8,
}

impl QoS {
    const D_FLAG: u8 = 0b00001000;
    const E_FLAG: u8 = 0b00010000;

    pub const DEFAULT: Self = Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false);
    pub const DECLARE: Self =
        Self::new(Priority::DEFAULT, CongestionControl::DEFAULT_DECLARE, false);

    pub const fn new(
        priority: Priority,
        congestion_control: CongestionControl,
        is_express: bool,
    ) -> Self {
        let mut inner = priority as u8;
        if matches!(congestion_control, CongestionControl::Block) {
            inner |= Self::D_FLAG;
        }
        if is_express {
            inner |= Self::E_FLAG;
        }
        Self { inner }
    }

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let inner: u8 = thread_rng().r#gen();
        Self { inner }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct NodeId {
    pub node_id: u16,
}

impl NodeId {
    pub const DEFAULT: Self = Self { node_id: 0 };

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let node_id: u16 = thread_rng().r#gen();
        Self { node_id }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum QueryTarget {
    #[default]
    BestMatching = 0,
    All = 1,
    AllComplete = 2,
}

impl QueryTarget {
    pub const DEFAULT: Self = Self::BestMatching;

    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> Self {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();

        *[
            QueryTarget::All,
            QueryTarget::AllComplete,
            QueryTarget::BestMatching,
        ]
        .choose(&mut rng)
        .unwrap()
    }
}

impl ZBodyLen for QueryTarget {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&((*self as u8) as u64))
    }
}

impl ZBodyEncode for QueryTarget {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZEncode>::z_encode(&((*self as u8) as u64), w)
    }
}

impl ZBodyDecode<'_> for QueryTarget {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'_>, _: ()) -> ZCodecResult<Self> {
        let value = <u64 as ZDecode>::z_decode(r)?;

        match value as u8 {
            0 => Ok(QueryTarget::BestMatching),
            1 => Ok(QueryTarget::All),
            2 => Ok(QueryTarget::AllComplete),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

crate::derive_zstruct_with_body!(QueryTarget);

impl<'a> ZExt<'a> for QueryTarget {
    const KIND: ZExtKind = ZExtKind::U64;
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Budget {
    pub budget: u32,
}

impl Budget {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let budget: u32 = thread_rng().r#gen();
        Self { budget }
    }
}

impl ZHeader for Duration {
    fn z_header(&self) -> u8 {
        let header = 0u8;
        match self.as_millis() % 1_000 {
            0 => header | 0b0000_0001,
            _ => header,
        }
    }
}

impl ZBodyLen for Duration {
    fn z_body_len(&self) -> usize {
        let v = match self.as_millis() % 1_000 {
            0 => self.as_millis() / 1_000,
            _ => self.as_millis(),
        } as u64;

        <u64 as ZLen>::z_len(&v)
    }
}

impl ZLen for Duration {
    fn z_len(&self) -> usize {
        <u64 as ZLen>::z_len(&(self.as_millis() as u64))
    }
}

impl ZBodyEncode for Duration {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        let v = match self.as_millis() % 1_000 {
            0 => self.as_millis() / 1_000,
            _ => self.as_millis(),
        } as u64;

        <u64 as ZEncode>::z_encode(&v, w)
    }
}

impl ZEncode for Duration {
    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZEncode>::z_encode(&(self.as_millis() as u64), w)
    }
}

impl<'a> ZBodyDecode<'a> for Duration {
    type Ctx = u8;

    fn z_body_decode(r: &mut ZReader<'a>, h: u8) -> ZCodecResult<Self> {
        let is_seconds = (h & 0b0000_0001) != 0;
        let value = <u64 as ZDecode>::z_decode(r)?;
        if is_seconds {
            Ok(Duration::from_secs(value))
        } else {
            Ok(Duration::from_millis(value))
        }
    }
}

impl<'a> ZDecode<'a> for Duration {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        Ok(Duration::from_millis(value))
    }
}

impl<'a> ZExt<'a> for Duration {
    const KIND: ZExtKind = ZExtKind::U64;
}

#[derive(Debug, PartialEq)]
pub struct QueryableInfo {
    pub complete: bool,
    pub distance: u16,
}

impl ZBodyLen for QueryableInfo {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&self.as_u64())
    }
}

impl ZBodyEncode for QueryableInfo {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZEncode>::z_encode(&self.as_u64(), w)
    }
}

impl ZBodyDecode<'_> for QueryableInfo {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'_>, _: ()) -> ZCodecResult<Self> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        let complete = (value & 0b0000_0001) != 0;
        let distance = ((value >> 8) & 0xFFFF) as u16;
        Ok(QueryableInfo { complete, distance })
    }
}

crate::derive_zstruct_with_body!(QueryableInfo);

impl<'a> ZExt<'a> for QueryableInfo {
    const KIND: ZExtKind = ZExtKind::U64;
}

impl QueryableInfo {
    pub const DEFAULT: Self = Self {
        complete: false,
        distance: 0,
    };

    fn as_u64(&self) -> u64 {
        let mut flags: u8 = 0;
        if self.complete {
            flags |= 0b0000_0001;
        }
        (flags as u64) | ((self.distance as u64) << 8)
    }

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let complete = thread_rng().gen_bool(0.5);
        let distance: u16 = thread_rng().r#gen();
        Self { complete, distance }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

impl From<Mapping> for u8 {
    fn from(val: Mapping) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for Mapping {
    type Error = ZCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mapping::Receiver),
            1 => Ok(Mapping::Sender),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

impl Mapping {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        if thread_rng().gen_bool(0.5) {
            Mapping::Receiver
        } else {
            Mapping::Sender
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq)]
pub enum Priority {
    #[default]
    Data = 5,
}

impl Priority {
    pub const DEFAULT: Self = Self::Data;
}

#[derive(Debug, Default, PartialEq)]
#[repr(u8)]
pub enum CongestionControl {
    #[default]
    Drop = 0,
    Block = 1,
}

impl CongestionControl {
    pub const DEFAULT: Self = Self::Drop;
    pub const DEFAULT_DECLARE: Self = Self::Block;
}
