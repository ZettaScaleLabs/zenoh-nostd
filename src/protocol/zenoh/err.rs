use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_zbuf, encode_u8, encode_zbuf},
        core::encoding::{Encoding, decode_encoding, encode_encoding},
        ext::{decode_ext_header, skip_ext},
        exts::{SourceInfo, decode_source_info, encode_source_info},
        has_flag,
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufReader, ZBufWriter},
    zext_id,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Err<'a> {
    // --- Body ---
    pub(crate) payload: ZBuf<'a>,

    // --- Optional Body that appears in flags ---
    pub(crate) encoding: Option<Encoding<'a>>,

    // --- Extensions ---
    pub(crate) ext_sinfo: Option<SourceInfo>,
}

impl<'a> Err<'a> {
    pub(crate) const ID: u8 = 5;

    const FLAG_E: u8 = 1 << 6;
    const FLAG_Z: u8 = 1 << 7;

    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;

        if self.encoding.is_some() {
            header |= Self::FLAG_E;
        }

        if self.ext_sinfo.is_some() {
            header |= Self::FLAG_Z;
        }

        encode_u8(writer, header)?;

        if let Some(encoding) = self.encoding.as_ref() {
            encode_encoding(writer, encoding)?;
        }

        encode_source_info::<Self>(writer, self.ext_sinfo.as_ref(), false)?;

        encode_zbuf(writer, self.payload, true)
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        let mut encoding = Option::<Encoding>::None;

        if has_flag(header, Self::FLAG_E) {
            encoding = Some(decode_encoding(reader)?);
        }

        let mut ext_sinfo: Option<SourceInfo> = None;

        let mut has_ext = has_flag(header, Self::FLAG_Z);
        while has_ext {
            let (id, kind, mandatory, more) = decode_ext_header(reader)?;
            has_ext = more;

            match id {
                zext_id!(SourceInfo) => {
                    ext_sinfo = Some(decode_source_info::<Self>(reader)?);
                }
                _ => {
                    skip_ext(reader, kind)?;

                    if mandatory {
                        crate::warn!(
                            "Mandatory extension with id {} in Err message not supported.",
                            id
                        );
                    }
                }
            }
        }

        let payload = decode_zbuf(reader, None)?;

        Ok(Err {
            encoding,
            ext_sinfo,

            payload,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::zbuf::BufWriterExt;

        let mut rng = rand::thread_rng();

        let encoding = rng.gen_bool(0.5).then_some(Encoding::rand(zbuf));
        let ext_sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let payload = zbuf
            .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self {
            encoding,
            ext_sinfo,
            payload,
        }
    }
}

crate::zext!(impl<'a> SourceInfo, Err<'a>, 0x1, false);
