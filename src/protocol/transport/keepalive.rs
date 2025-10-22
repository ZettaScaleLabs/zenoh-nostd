use crate::{
    protocol::{
        ZCodecError,
        codec::encode_u8,
        common::{extension, imsg},
        transport::id,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {

    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeepAlive;

impl KeepAlive {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        encode_u8(writer, id::KEEP_ALIVE)?;
        Ok(())
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'_>, header: u8) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::KEEP_ALIVE {
            zbail!(ZCodecError::CouldNotRead)
        }

        let has_ext = imsg::has_flag(header, flag::Z);
        if has_ext {
            extension::skip_all("Unknown KeepAlive ext", reader)?;
        }

        Ok(KeepAlive)
    }

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZBufWriter<'_>) -> Self {
        Self
    }
}
