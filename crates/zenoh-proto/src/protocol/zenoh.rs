use crate::{
    Encoding, ZBodyDecode, ZBodyEncode, ZBodyLen, ZCodecResult, ZDecode, ZEncode, ZEnum, ZExt,
    ZLen, ZReader, ZWriter, ZenohIdProto,
};

#[cfg(test)]
use {
    crate::ZWriterExt,
    rand::{Rng, seq::SliceRandom, thread_rng},
};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

use err::Err;
use put::Put;
use query::Query;
use reply::Reply;

#[derive(ZEnum, Debug, PartialEq)]
pub enum PushBody<'a> {
    Put(Put<'a>),
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum RequestBody<'a> {
    Query(Query<'a>),
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum ResponseBody<'a> {
    Err(Err<'a>),
    Reply(Reply<'a>),
}

#[derive(ZExt, Debug, PartialEq)]
#[zenoh(header = "ID:4|_:4")]
pub struct EntityGlobalId {
    #[zenoh(size = header(ID))]
    pub zid: ZenohIdProto,

    pub eid: u32,
}

impl EntityGlobalId {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter) -> Self {
        let zid = ZenohIdProto::rand(w);
        let eid: u32 = thread_rng().r#gen();

        Self { zid, eid }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct SourceInfo {
    pub id: EntityGlobalId,
    pub sn: u32,
}

impl SourceInfo {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter) -> Self {
        let id = EntityGlobalId::rand(w);
        let sn: u32 = thread_rng().r#gen();

        Self { id, sn }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Value<'a> {
    pub encoding: Encoding<'a>,

    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

impl<'a> Value<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let encoding = Encoding::rand(w);
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { encoding, payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Attachment<'a> {
    #[zenoh(size = remain)]
    pub buffer: &'a [u8],
}

impl<'a> Attachment<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let buffer = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { buffer }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Copy)]
pub enum ConsolidationMode {
    #[default]
    Auto = 0,
    None = 1,
    Monotonic = 2,
    Latest = 3,
}

impl ConsolidationMode {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        *[Self::None, Self::Monotonic, Self::Latest, Self::Auto]
            .choose(&mut thread_rng())
            .unwrap()
    }
}

impl ZBodyLen for ConsolidationMode {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&((*self as u8) as u64))
    }
}

impl ZBodyEncode for ConsolidationMode {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZEncode>::z_encode(&((*self as u8) as u64), w)
    }
}

impl<'a> ZBodyDecode<'a> for ConsolidationMode {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> ZCodecResult<Self> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        match value as u8 {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

crate::derive_zstruct_with_body!(ConsolidationMode);
