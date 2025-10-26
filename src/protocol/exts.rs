use zenoh_proto::ZExt;

#[cfg(test)]
use crate::zbuf::ZBufWriter;
use crate::{
    protocol::{
        codec::{
            decode_u8, decode_u32, decode_usize, decode_zbuf, decode_zid, encode_u8, encode_u32,
            encode_usize, encode_zbuf, encode_zid, encoded_len_u32, encoded_len_zbuf,
            encoded_len_zid,
        },
        core::{
            EntityId, ZenohIdProto,
            encoding::{Encoding, decode_encoding, encode_encoding, encoded_len_encoding},
        },
        ext::ZExtKind,
    },
    zbuf::{BufReaderExt, ZBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceInfo {
    // It should be possible to use the `deduce` flavor if we reorder the fields but for backward compatibility we keep the current order.
    // #[zid(flag = 4)]
    pub(crate) zid: ZenohIdProto,

    // #[u32]
    pub(crate) eid: EntityId,

    // #[u32]
    pub(crate) sn: u32,
}

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

crate::zext!(
    'static,
    SourceInfo,
    ZExtKind::ZBuf,
    |s| {
        1 + encoded_len_zid(&s.zid) + encoded_len_u32(s.eid) + encoded_len_u32(s.sn)
    },
    |w, x| {
        let flags: u8 = (x.zid.size() as u8 - 1) << 4;
        encode_u8(w, flags)?;

        encode_zid(w, &x.zid)?;
        encode_u32(w, x.eid)?;
        encode_u32(w, x.sn)
    },
    |r, _l| {
        let flags = decode_u8(r)?;
        let len = 1 + ((flags >> 4) as usize);

        let zid = decode_zid(r, len)?;
        let eid = decode_u32(r)?;
        let sn = decode_u32(r)?;

        Ok(SourceInfo { zid, eid, sn })
    }
);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Value<'a> {
    // #[composite(encoding)]
    pub(crate) encoding: Encoding<'a>,

    // #[zbuf(deduce)]
    pub(crate) payload: ZBuf<'a>,
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

crate::zext!(
    'a,
    Value,
    ZExtKind::ZBuf,
    |s| {
        encoded_len_encoding(&s.encoding) + encoded_len_zbuf(&s.payload)
    },
    |w, x| {
        encode_encoding(w, &x.encoding)?;
        encode_zbuf(w, x.payload)
    },
    |r, len| {
        let start = r.remaining();
        let encoding = decode_encoding(r)?;
        let end = r.remaining();

        let payload_len = len - (start - end);
        let payload = decode_zbuf(r, payload_len)?;

        Ok(Value { encoding, payload })
    }
);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Attachment<'a> {
    // #[zbuf(deduce)]
    pub(crate) buffer: ZBuf<'a>,
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

crate::zext!(
    'a,
    Attachment,
    ZExtKind::ZBuf,
    |s| {
        encoded_len_zbuf(&s.buffer)
    },
    |w, x| {
        encode_zbuf(w, &x.buffer)
    },
    |r, len| {
        let buffer = decode_zbuf(r, len)?;
        Ok(Attachment { buffer })
    }
);
