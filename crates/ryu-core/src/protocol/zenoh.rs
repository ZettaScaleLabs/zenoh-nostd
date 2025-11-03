use ryu_derive::ZExt;

#[cfg(test)]
use crate::ByteWriter;
use crate::core::{ZenohIdProto, encoding::Encoding};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

// TODO for v2: **zid** should be put at the end with a **deduced** flavour. This would reduce 1 byte of
// overhead.
#[derive(ZExt, Debug, PartialEq)]
pub struct EntityGlobalId {
    _flag: crate::marker::Flag,

    #[size(flag = 4)]
    pub zid: ZenohIdProto,

    pub eid: u32,
}

impl EntityGlobalId {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let zid = ZenohIdProto::rand();
        let eid: u32 = rng.r#gen();

        Self {
            _flag: crate::marker::Flag,
            zid,
            eid,
        }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct SourceInfo {
    pub id: EntityGlobalId, // used as a regular ZStruct
    pub sn: u32,
}

impl SourceInfo {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let id = EntityGlobalId::rand();
        let sn: u32 = rng.r#gen();

        Self { id, sn }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Value<'a> {
    #[size(none)] // Self bounded
    pub encoding: Encoding<'a>,

    #[size(deduced)]
    pub payload: &'a [u8],
}

impl<'a> Value<'a> {
    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> Self {
        use rand::Rng;

        use crate::ByteWriterExt;

        let mut rng = rand::thread_rng();

        let encoding = Encoding::rand(zbuf);
        let payload = zbuf
            .write_slot(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self { encoding, payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Attachment<'a> {
    #[size(deduced)]
    pub buffer: &'a [u8],
}
