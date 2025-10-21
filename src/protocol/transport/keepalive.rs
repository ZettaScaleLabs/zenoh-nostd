use crate::{
    protocol::{
        ZCodecError,
        common::{extension, imsg},
        transport::id,
        zcodec::encode_u8,
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

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
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
    pub(crate) fn rand() -> Self {
        Self
    }
}
