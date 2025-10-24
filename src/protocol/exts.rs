#[cfg(test)]
use crate::zbuf::ZBufWriter;
use crate::{
    protocol::{
        codec::{
            decode_u8, decode_u32, decode_usize, decode_zbuf, encode_u8, encode_u32, encode_usize,
            encode_zbuf, encoded_len_u32, encoded_len_zbuf,
        },
        core::{
            EntityId, ZenohIdProto, decode_zid, encode_zid, encoded_len_zid,
            encoding::{Encoding, decode_encoding, encode_encoding, encoded_len_encoding},
        },
        ext::ZExtKind,
    },
    zbuf::{BufReaderExt, ZBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceInfo {
    pub(crate) zid: ZenohIdProto,
    pub(crate) eid: EntityId,
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
    SourceInfo,
    ZExtKind::ZBuf,
    |w, x| {
        let len = 1 + encoded_len_zid(&x.zid) + encoded_len_u32(x.eid) + encoded_len_u32(x.sn);
        encode_usize(w, len)?;

        let flags: u8 = (x.zid.size() as u8 - 1) << 4;
        encode_u8(w, flags)?;

        encode_zid(w, &x.zid)?;
        encode_u32(w, x.eid)?;
        encode_u32(w, x.sn)
    },
    |r| {
        let _ = decode_usize(r)?;

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
    pub(crate) encoding: Encoding<'a>,
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
    Value<'a>,
    ZExtKind::ZBuf,
    |w, x| {
        let len = encoded_len_encoding(&x.encoding) + encoded_len_zbuf(&x.payload);
        encode_usize(w, len)?;

        encode_encoding(w, &x.encoding)?;
        encode_zbuf(w, x.payload)
    },
    |r| {
        let len = decode_usize(r)?;

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
    Attachment<'a>,
    ZExtKind::ZBuf,
    |w, x| {
        let len = encoded_len_zbuf(&x.buffer);
        encode_usize(w, len)?;

        encode_zbuf(w, &x.buffer)
    },
    |r| {
        let len = decode_usize(r)?;

        let buffer = decode_zbuf(r, len)?;
        Ok(Attachment { buffer })
    }
);
