use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_str, decode_u8, encode_str, encode_u8, encode_u64},
        ext::{decode_ext_header, skip_ext},
        exts::{
            Attachment, SourceInfo, Value, decode_attachment, decode_source_info, decode_value,
            encode_attachment, encode_source_info, encode_value,
        },
        has_flag, msg_id,
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
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Query<'a> {
    pub(crate) consolidation: ConsolidationMode,
    pub(crate) parameters: &'a str,

    pub(crate) ext_sinfo: Option<SourceInfo>,
    pub(crate) ext_body: Option<Value<'a>>,
    pub(crate) ext_attachment: Option<Attachment<'a>>,
}

impl<'a> Query<'a> {
    #[allow(unused_assignments)]
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
            encode_str(writer, self.parameters, true)?;
        }

        n_exts -=
            encode_source_info::<Self>(writer, &self.ext_sinfo, n_exts > 1 && (n_exts - 1) > 0)?
                as u8;

        n_exts -=
            encode_value::<Self>(writer, &self.ext_body, n_exts > 1 && (n_exts - 1) > 0)? as u8;

        n_exts -= encode_attachment::<Self>(
            writer,
            &self.ext_attachment,
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        Ok(())
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        if msg_id(header) != id::QUERY {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if has_flag(header, flag::C) {
            consolidation = ConsolidationMode::decode(reader)?;
        }

        let mut parameters = "";
        if has_flag(header, flag::P) {
            parameters = decode_str(reader, None)?;
        }

        let mut ext_sinfo: Option<SourceInfo> = None;
        let mut ext_body: Option<Value> = None;
        let mut ext_attachment: Option<Attachment> = None;

        let mut has_ext = has_flag(header, flag::Z);
        while has_ext {
            let (id, kind, mandatory, more) = decode_ext_header(reader)?;
            has_ext = more;

            match id {
                crate::zext_id!(SourceInfo) => {
                    ext_sinfo = Some(decode_source_info::<Self>(reader)?);
                }
                crate::zext_id!(Value) => {
                    ext_body = Some(decode_value::<Self>(reader)?);
                }
                crate::zext_id!(Attachment) => {
                    ext_attachment = Some(decode_attachment::<Self>(reader)?);
                }
                _ => {
                    skip_ext(reader, kind)?;

                    if mandatory {
                        crate::warn!(
                            "Mandatory extension with id {} in Query message not supported.",
                            id
                        );
                    }
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

        let ext_sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let ext_body = rng.gen_bool(0.5).then_some(Value::rand(zbuf));
        let ext_attachment = rng.gen_bool(0.5).then_some(Attachment::rand(zbuf));

        Self {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
        }
    }
}

crate::zext!(impl<'a> SourceInfo, Query<'a>, 0x1, false);
crate::zext!(impl<'a> Value<'a>, Query<'a>, 0x3, false);
crate::zext!(impl<'a> Attachment<'a>, Query<'a>, 0x5, false);
