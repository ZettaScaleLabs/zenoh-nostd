use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_str, decode_usize, encode_str, encode_u8, encode_usize},
        ext::{decode_ext_header, skip_ext},
        exts::{
            Attachment, SourceInfo, Value, decode_attachment, decode_source_info, decode_value,
            encode_attachment, encode_source_info, encode_value,
        },
        has_flag,
        zenoh::{ConsolidationMode, decode_consolidation_mode, encode_consolidation_mode},
    },
    result::ZResult,
    zbuf::{ZBufReader, ZBufWriter},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Query<'a> {
    // --- Optional Body that appears in flags ---
    pub(crate) consolidation: Option<ConsolidationMode>,
    pub(crate) parameters: Option<&'a str>,

    // --- Extensions ---
    pub(crate) ext_sinfo: Option<SourceInfo>,
    pub(crate) ext_body: Option<Value<'a>>,
    pub(crate) ext_attachment: Option<Attachment<'a>>,
}

impl<'a> Query<'a> {
    pub(crate) const ID: u8 = 0x03;

    const FLAG_C: u8 = 1 << 5;
    const FLAG_P: u8 = 1 << 6;
    const FLAG_Z: u8 = 1 << 7;

    #[allow(unused_assignments)]
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;

        if self.consolidation.is_some() {
            header |= Self::FLAG_C;
        }

        if self.parameters.is_some() {
            header |= Self::FLAG_P;
        }

        let mut n_exts = (self.ext_sinfo.is_some() as u8)
            + (self.ext_body.is_some() as u8)
            + (self.ext_attachment.is_some() as u8);

        if n_exts != 0 {
            header |= Self::FLAG_Z;
        }

        encode_u8(writer, header)?;

        if let Some(consolidation) = self.consolidation.as_ref() {
            encode_consolidation_mode(writer, consolidation)?;
        }

        if let Some(params) = self.parameters.as_ref() {
            encode_usize(writer, params.len())?;
            encode_str(writer, params)?;
        }

        n_exts -= encode_source_info::<Self>(
            writer,
            self.ext_sinfo.as_ref(),
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        n_exts -= encode_value::<Self>(
            writer,
            self.ext_body.as_ref(),
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        n_exts -= encode_attachment::<Self>(
            writer,
            self.ext_attachment.as_ref(),
            n_exts > 1 && (n_exts - 1) > 0,
        )? as u8;

        Ok(())
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        let mut consolidation = Option::<ConsolidationMode>::None;
        if has_flag(header, Self::FLAG_C) {
            consolidation = Some(decode_consolidation_mode(reader)?);
        }

        let mut parameters = Option::<&'a str>::None;
        if has_flag(header, Self::FLAG_P) {
            let len = decode_usize(reader)?;
            parameters = Some(decode_str(reader, len)?);
        }

        let mut ext_sinfo: Option<SourceInfo> = None;
        let mut ext_body: Option<Value> = None;
        let mut ext_attachment: Option<Attachment> = None;

        let mut has_ext = has_flag(header, Self::FLAG_Z);
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

        let consolidation = rng.gen_bool(0.5).then_some(ConsolidationMode::rand());
        let parameters = rng.gen_bool(0.5).then(|| {
            use crate::zbuf::BufWriterExt;

            let len = rng.gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut rng, len);
            zbuf.write_str_return(proto.as_str()).unwrap()
        });

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
