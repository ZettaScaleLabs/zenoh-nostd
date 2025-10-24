use crate::{
    protocol::{
        ZCodecError,
        codec::{
            decode_timestamp, decode_usize, decode_zbuf, encode_timestamp, encode_u8, encode_usize,
            encode_zbuf,
        },
        core::encoding::{Encoding, decode_encoding, encode_encoding},
        ext::{decode_ext_header, skip_ext},
        exts::{
            Attachment, SourceInfo, decode_attachment, decode_source_info, encode_attachment,
            encode_source_info,
        },
        has_flag,
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufReader, ZBufWriter},
};

use uhlc::Timestamp;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Put<'a> {
    // --- Body ---
    pub(crate) payload: ZBuf<'a>,

    // --- Optional Body that appears in flags ---
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) encoding: Option<Encoding<'a>>,

    // --- Extensions ---
    pub(crate) ext_sinfo: Option<SourceInfo>,
    pub(crate) ext_attachment: Option<Attachment<'a>>,
}

impl<'a> Put<'a> {
    pub(crate) const ID: u8 = 0x01;

    pub(crate) const FLAG_T: u8 = 1 << 5;
    pub(crate) const FLAG_E: u8 = 1 << 6;
    pub(crate) const FLAG_Z: u8 = 1 << 7;
}

impl<'a> Put<'a> {
    #[allow(unused_assignments)]
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;

        if self.timestamp.is_some() {
            header |= Self::FLAG_T;
        }

        if self.encoding.is_some() {
            header |= Self::FLAG_E;
        }

        let mut n_exts = (self.ext_sinfo.is_some()) as u8 + (self.ext_attachment.is_some()) as u8;

        if n_exts != 0 {
            header |= Self::FLAG_Z;
        }

        encode_u8(writer, header)?;

        if let Some(ts) = self.timestamp.as_ref() {
            encode_timestamp(writer, ts)?;
        }

        if let Some(encoding) = self.encoding.as_ref() {
            encode_encoding(writer, encoding)?;
        }

        n_exts -= encode_source_info::<Self>(
            writer,
            self.ext_sinfo.as_ref(),
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        n_exts -= encode_attachment::<Self>(
            writer,
            self.ext_attachment.as_ref(),
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        encode_usize(writer, self.payload.len())?;
        encode_zbuf(writer, self.payload)
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        let mut timestamp: Option<uhlc::Timestamp> = None;

        if has_flag(header, Self::FLAG_T) {
            timestamp = Some(decode_timestamp(reader)?);
        }

        let mut encoding = Option::<Encoding>::None;
        if has_flag(header, Self::FLAG_E) {
            encoding = Some(decode_encoding(reader)?);
        }

        let mut ext_sinfo: Option<SourceInfo> = None;
        let mut ext_attachment: Option<Attachment> = None;

        let mut has_ext = has_flag(header, Self::FLAG_Z);
        while has_ext {
            let (id, kind, mandatory, more) = decode_ext_header(reader)?;
            has_ext = more;

            match id {
                crate::zext_id!(SourceInfo) => {
                    ext_sinfo = Some(decode_source_info::<Self>(reader)?);
                }
                crate::zext_id!(Attachment) => {
                    ext_attachment = Some(decode_attachment::<Self>(reader)?);
                }
                _ => {
                    skip_ext(reader, kind)?;

                    if mandatory {
                        crate::warn!(
                            "Mandatory extension with id {} in Put message not supported.",
                            id
                        );
                    }
                }
            }
        }

        let len = decode_usize(reader)?;
        let payload = decode_zbuf(reader, len)?;

        Ok(Put {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,

            payload,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::zbuf::BufWriterExt;

        let mut rng = rand::thread_rng();
        let timestamp = rng.gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(rng.r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });
        let encoding = rng.gen_bool(0.5).then_some(Encoding::rand(zbuf));
        let ext_sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let ext_attachment = rng.gen_bool(0.5).then_some(Attachment::rand(zbuf));
        let payload = zbuf
            .write_slot_return(rng.gen_range(1..=64), |b| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,
            payload,
        }
    }
}

crate::zext!(impl<'a> SourceInfo, Put<'a>, 0x1, false);
crate::zext!(impl<'a> Attachment<'a>, Put<'a>, 0x3, false);
