use zenoh_proto::ZExt;

#[cfg(test)]
use crate::zbuf::ZBufWriter;
use crate::{
    protocol::core::{EntityId, ZenohIdProto, encoding::Encoding},
    zbuf::ZBuf,
};

#[derive(ZExt, Debug, PartialEq, Eq)]
pub(crate) struct SourceInfo {
    // It should be possible to use the `deduce` flavour if we reorder the fields but for backward compatibility we keep the current one.
    #[zid(flag = 4)]
    pub(crate) zid: ZenohIdProto,

    #[u32]
    pub(crate) eid: EntityId,

    #[u32]
    pub(crate) sn: u32,
}

#[derive(ZExt, Debug, PartialEq, Eq)]
pub(crate) struct Value<'a> {
    #[composite(crate::protocol::core::encoding, encoding)]
    pub(crate) encoding: Encoding<'a>,

    #[zbuf(deduced)]
    pub(crate) payload: ZBuf<'a>,
}

#[derive(ZExt, Debug, PartialEq, Eq)]
pub(crate) struct Attachment<'a> {
    #[zbuf(deduced)]
    pub(crate) buffer: ZBuf<'a>,
}

crate::zext!('static, SourceInfo);
crate::zext!('a, Value);
crate::zext!('a, Attachment);

#[cfg(test)]
impl SourceInfo {
    pub(crate) fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            zid: ZenohIdProto::default(),
            eid: rng.r#gen(),
            sn: rng.r#gen(),
        }
    }
}

#[cfg(test)]
impl<'a> Value<'a> {
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::zbuf::BufWriterExt;
        let mut rng = rand::thread_rng();

        let encoding = Encoding::rand(zbuf);
        let payload = zbuf
            .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self { encoding, payload }
    }
}

#[cfg(test)]
impl<'a> Attachment<'a> {
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::zbuf::BufWriterExt;
        let mut rng = rand::thread_rng();
        let buffer = zbuf
            .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self { buffer }
    }
}
