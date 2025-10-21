use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::encoding::Encoding,
        zcodec::{
            decode_timestamp, decode_u8, decode_zbuf, encode_timestamp, encode_u8, encode_zbuf,
        },
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};
use uhlc::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Put<'a> {
    // ---------- Body for Put message ----------
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) encoding: Encoding<'a>,

    pub(crate) ext_sinfo: Option<ext::SourceInfoType>,
    pub(crate) ext_attachment: Option<ext::AttachmentType<'a>>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
    // ----------------------------------------
}

impl<'a> Put<'a> {
    // ---------- Header for Put message ----------

    /// Message ID for Put messages
    pub(crate) const ID: u8 = 0x01;

    /// Indicates that the Timestamp optional field is present
    pub(crate) const FLAG_T: u8 = 1 << 5;
    /// Indicates that the Encoding is not empty and should be present
    pub(crate) const FLAG_E: u8 = 1 << 6;
    /// Indicates that at least one extension is present
    pub(crate) const FLAG_Z: u8 = 1 << 7;

    // ---------------------------------------------
}

impl<'a> Put<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;

        if self.timestamp.is_some() {
            header |= Self::FLAG_T;
        }

        if self.encoding != Encoding::empty() {
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

        encode_zbuf(writer, true, self.payload)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != Self::ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut timestamp: Option<uhlc::Timestamp> = None;
        if imsg::has_flag(header, Self::FLAG_T) {
            timestamp = Some(decode_timestamp(reader)?);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(header, Self::FLAG_E) {
            encoding = Encoding::decode(reader)?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;

        let mut has_ext = imsg::has_flag(header, Self::FLAG_Z);
        while has_ext {
            let ext = decode_u8(reader)?;

            match iext::eheader(ext) {
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

        let payload = decode_zbuf(reader, None)?;

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
            let id = uhlc::ID::try_from(ZenohIdProto::rand().as_le_bytes()).unwrap();
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
