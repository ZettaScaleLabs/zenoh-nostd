use crate::ZExt;
use crate::core::{ZenohIdProto, encoding::Encoding};

#[cfg(test)]
use crate::{ZWriter, ZWriterExt};
#[cfg(test)]
use rand::{Rng, thread_rng};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

use err::Err;
use put::Put;
use query::Query;
use reply::Reply;

crate::__internal_zaggregate! {
    #[derive(Debug, PartialEq)]
    pub enum PushBody<'a> {
        Put<'a>,
    }
}

crate::__internal_zaggregate! {
    #[derive(Debug, PartialEq)]
    pub enum RequestBody<'a> {
        Query<'a>,
    }
}

crate::__internal_zaggregate! {
    #[derive(Debug, PartialEq)]
    pub enum ResponseBody<'a> {
        Err<'a>,
        Reply<'a>,
    }
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
