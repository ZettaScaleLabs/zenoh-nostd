use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::encoding::Encoding,
        zcodec::{decode_timestamp, decode_zbuf, encode_timestamp, encode_zbuf},
        zenoh::id,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};
use uhlc::Timestamp;

pub(crate) mod flag {
    pub(crate) const T: u8 = 1 << 5;
    pub(crate) const E: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Put<'a> {
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) encoding: Encoding<'a>,
    pub(crate) ext_sinfo: Option<ext::SourceInfoType>,
    pub(crate) ext_attachment: Option<ext::AttachmentType<'a>>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
}

impl<'a> Put<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::PUT;

        if self.timestamp.is_some() {
            header |= flag::T;
        }

        if self.encoding != Encoding::empty() {
            header |= flag::E;
        }

        let mut n_exts = (self.ext_sinfo.is_some()) as u8 + (self.ext_attachment.is_some()) as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }

        crate::protocol::zcodec::encode_u8(header, writer)?;

        if let Some(ts) = self.timestamp.as_ref() {
            encode_timestamp(ts, writer)?;
        }

        if self.encoding != Encoding::empty() {
            self.encoding.encode(writer)?;
        }

        if let Some(sinfo) = self.ext_sinfo.as_ref() {
            n_exts -= 1;
            sinfo.encode(n_exts != 0, writer)?;
        }

        if let Some(att) = self.ext_attachment.as_ref() {
            n_exts -= 1;
            att.encode(n_exts != 0, writer)?;
        }

        encode_zbuf(true, self.payload, writer)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::PUT {
            zbail!(ZCodecError::Invalid);
        }

        let mut timestamp: Option<uhlc::Timestamp> = None;
        if imsg::has_flag(header, flag::T) {
            timestamp = Some(decode_timestamp(reader)?);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(header, flag::E) {
            encoding = Encoding::decode(reader)?;
        }

        // Extensions
        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext = crate::protocol::zcodec::decode_u8(reader)?;

            match iext::eid(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext) = ext::SourceInfoType::decode(ext, reader)?;

                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                ext::Attachment::ID => {
                    let (a, ext) = ext::AttachmentType::decode(ext, reader)?;
                    ext_attachment = Some(a);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Put", ext, reader)?;
                }
            }
        }

        let payload = decode_zbuf(None, reader)?;

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
            let id = uhlc::ID::try_from(ZenohIdProto::rand().to_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });
        let encoding = Encoding::rand(zbuf);
        let ext_sinfo = rng.gen_bool(0.5).then_some(ext::SourceInfoType::rand());
        let ext_attachment = rng.gen_bool(0.5).then_some(ext::AttachmentType::rand(zbuf));
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

pub(crate) mod ext {
    pub(crate) type SourceInfo<'a> = crate::zextzbuf!('a, 0x1, false);
    pub(crate) type SourceInfoType =
        crate::protocol::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;

    pub(crate) type Attachment<'a> = crate::zextzbuf!('a, 0x3, false);
    pub(crate) type AttachmentType<'a> =
        crate::protocol::zenoh::ext::AttachmentType<'a, { Attachment::ID }>;
}
