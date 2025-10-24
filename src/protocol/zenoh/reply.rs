use crate::{
    protocol::{
        ZCodecError,
        codec::encode_u8,
        ext::{decode_ext_header, skip_ext},
        has_flag, msg_id,
        zenoh::{
            ConsolidationMode, PushBody, decode_consolidation_mode, encode_consolidation_mode,
        },
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Reply<'a> {
    // --- Body ---
    pub(crate) payload: ReplyBody<'a>,

    // --- Optional Body that appears in flags ---
    pub(crate) consolidation: Option<ConsolidationMode>,
}

pub(crate) type ReplyBody<'a> = PushBody<'a>;

impl<'a> Reply<'a> {
    pub(crate) const ID: u8 = 0x04;

    const FLAG_C: u8 = 1 << 5;
    const FLAG_Z: u8 = 1 << 7;

    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;

        if self.consolidation.is_some() {
            header |= Self::FLAG_C;
        }

        encode_u8(writer, header)?;

        if let Some(consolidation) = self.consolidation.as_ref() {
            encode_consolidation_mode(writer, consolidation)?;
        }

        self.payload.encode(writer)
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        if msg_id(header) != Self::ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut consolidation = Option::<ConsolidationMode>::None;
        if has_flag(header, Self::FLAG_C) {
            consolidation = Some(decode_consolidation_mode(reader)?);
        }

        let mut has_ext = has_flag(header, Self::FLAG_Z);
        while has_ext {
            let (id, kind, mandatory, more) = decode_ext_header(reader)?;
            has_ext = more;
            skip_ext(reader, kind)?;

            if mandatory {
                crate::warn!(
                    "Mandatory extension with id {} in Reply message not supported.",
                    id
                );
            }
        }

        let payload = ReplyBody::decode(reader)?;

        Ok(Reply {
            consolidation,
            payload,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::{Rng, thread_rng};

        let payload = ReplyBody::rand(zbuf);
        let consolidation = thread_rng()
            .gen_bool(0.5)
            .then_some(ConsolidationMode::rand());

        Self {
            consolidation,
            payload,
        }
    }
}
