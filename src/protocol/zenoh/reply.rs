use crate::{
    protocol::{
        ZCodecError,
        common::{extension, imsg},
        zcodec::encode_u8,
        zenoh::{PushBody, id, query::ConsolidationMode},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const C: u8 = 1 << 5;

    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reply<'a> {
    pub(crate) consolidation: ConsolidationMode,
    pub(crate) payload: ReplyBody<'a>,
}

pub(crate) type ReplyBody<'a> = PushBody<'a>;

impl<'a> Reply<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::REPLY;

        if self.consolidation != ConsolidationMode::DEFAULT {
            header |= flag::C;
        }

        encode_u8(writer, header)?;

        if self.consolidation != ConsolidationMode::DEFAULT {
            self.consolidation.encode(writer)?;
        }

        self.payload.encode(writer)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::REPLY {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = ConsolidationMode::decode(reader)?;
        }

        if imsg::has_flag(header, flag::Z) {
            extension::skip_all("Reply", reader)?;
        }

        let payload = ReplyBody::decode(reader)?;

        Ok(Reply {
            consolidation,
            payload,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        let payload = ReplyBody::rand(zbuf);
        let consolidation = ConsolidationMode::rand();

        Self {
            consolidation,
            payload,
        }
    }
}
