use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::encoding::Encoding,
        zcodec::{decode_u8, decode_zbuf, encode_u8, encode_zbuf},
        zenoh::id,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {

    pub(crate) const E: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Err<'a> {
    pub(crate) encoding: Encoding<'a>,
    pub(crate) ext_sinfo: Option<ext::SourceInfoType>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
}

impl<'a> Err<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::ERR;
        if self.encoding != Encoding::empty() {
            header |= flag::E;
        }

        let mut n_exts = self.ext_sinfo.is_some() as u8;
        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;

        if self.encoding != Encoding::empty() {
            self.encoding.encode(writer)?;
        }

        if let Some(sinfo) = self.ext_sinfo.as_ref() {
            n_exts -= 1;
            sinfo.encode(n_exts != 0, writer)?;
        }

        encode_zbuf(writer, true, self.payload)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::ERR {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(header, flag::E) {
            encoding = Encoding::decode(reader)?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext) = ext::SourceInfoType::decode(ext, reader)?;

                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Err", ext, reader)?;
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

        let encoding = Encoding::rand(zbuf);
        let ext_sinfo = rng.gen_bool(0.5).then_some(ext::SourceInfoType::rand());
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

pub(crate) mod ext {

    pub(crate) type SourceInfo<'a> = crate::zextzbuf!('a, 0x1, false);
    pub(crate) type SourceInfoType =
        crate::protocol::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;
}
