use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        zcodec::{decode_str, decode_u8, encode_str, encode_u8, encode_u64},
        zenoh::id,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub(crate) enum ConsolidationMode {
    #[default]
    Auto,
    None,
    Monotonic,
    Latest,
}

impl ConsolidationMode {
    pub(crate) const DEFAULT: Self = Self::Auto;

    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let x: u64 = match self {
            ConsolidationMode::Auto => 0,
            ConsolidationMode::None => 1,
            ConsolidationMode::Monotonic => 2,
            ConsolidationMode::Latest => 3,
        };

        encode_u64(writer, x)
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
        let x = decode_u8(reader)?;

        match x {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Ok(ConsolidationMode::Auto),
        }
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::prelude::SliceRandom;
        let mut rng = rand::thread_rng();

        *[Self::None, Self::Monotonic, Self::Latest, Self::Auto]
            .choose(&mut rng)
            .unwrap()
    }
}

pub(crate) mod flag {
    pub(crate) const C: u8 = 1 << 5;
    pub(crate) const P: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Query<'a> {
    pub(crate) consolidation: ConsolidationMode,
    pub(crate) parameters: &'a str,
    pub(crate) ext_sinfo: Option<ext::SourceInfoType>,
    pub(crate) ext_body: Option<ext::QueryBodyType<'a>>,
    pub(crate) ext_attachment: Option<ext::AttachmentType<'a>>,
}

impl<'a> Query<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::QUERY;

        if self.consolidation != ConsolidationMode::DEFAULT {
            header |= flag::C;
        }

        if !self.parameters.is_empty() {
            header |= flag::P;
        }

        let mut n_exts = (self.ext_sinfo.is_some() as u8)
            + (self.ext_body.is_some() as u8)
            + (self.ext_attachment.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;

        if self.consolidation != ConsolidationMode::DEFAULT {
            self.consolidation.encode(writer)?;
        }

        if !self.parameters.is_empty() {
            encode_str(writer, true, self.parameters)?;
        }

        if let Some(sinfo) = self.ext_sinfo.as_ref() {
            n_exts -= 1;
            sinfo.encode(n_exts != 0, writer)?;
        }

        if let Some(body) = self.ext_body.as_ref() {
            n_exts -= 1;
            body.encode(n_exts != 0, writer)?;
        }

        if let Some(att) = self.ext_attachment.as_ref() {
            n_exts -= 1;
            att.encode(n_exts != 0, writer)?;
        }

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::QUERY {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = ConsolidationMode::decode(reader)?;
        }

        let mut parameters = "";
        if imsg::has_flag(header, flag::P) {
            parameters = decode_str(reader, None)?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_body: Option<ext::QueryBodyType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext = decode_u8(reader)?;

            match iext::eheader(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext) = ext::SourceInfoType::decode(ext, reader)?;

                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                ext::QueryBodyType::SID | ext::QueryBodyType::VID => {
                    let (s, ext) = ext::QueryBodyType::decode(ext, reader)?;

                    ext_body = Some(s);
                    has_ext = ext;
                }
                ext::Attachment::ID => {
                    let (a, ext) = ext::AttachmentType::decode(ext, reader)?;

                    ext_attachment = Some(a);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Query", ext, reader)?;
                }
            }
        }

        Ok(Query {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::{
            Rng,
            distributions::{Alphanumeric, DistString},
        };

        let mut rng = rand::thread_rng();

        const MIN: usize = 0;
        const MAX: usize = 16;

        let consolidation = ConsolidationMode::rand();
        let parameters = if rng.gen_bool(0.5) {
            use crate::zbuf::BufWriterExt;

            let len = rng.gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut rng, len);
            zbuf.write_str_return(proto.as_str()).unwrap()
        } else {
            ""
        };
        let ext_sinfo = rng.gen_bool(0.5).then_some(ext::SourceInfoType::rand());
        let ext_body = rng.gen_bool(0.5).then_some(ext::QueryBodyType::rand(zbuf));
        let ext_attachment = rng.gen_bool(0.5).then_some(ext::AttachmentType::rand(zbuf));

        Self {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
        }
    }
}

pub(crate) mod ext {
    use crate::protocol::common::extension::ZExtZBuf;

    pub(crate) type SourceInfo<'a> = crate::zextzbuf!('a, 0x1, false);
    pub(crate) type SourceInfoType =
        crate::protocol::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;

    pub(crate) type QueryBodyType<'a> =
        crate::protocol::zenoh::ext::ValueType<'a, { ZExtZBuf::<0x03>::id(false) }, 0x04>;

    pub(crate) type Attachment<'a> = crate::zextzbuf!('a, 0x5, false);
    pub(crate) type AttachmentType<'a> =
        crate::protocol::zenoh::ext::AttachmentType<'a, { Attachment::ID }>;
}
