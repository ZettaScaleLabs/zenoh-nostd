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

pub mod flag {

    pub const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeepAlive;

impl KeepAlive {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        encode_u8(id::KEEP_ALIVE, writer)?;
        Ok(())
    }

    pub fn decode(header: u8, reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::KEEP_ALIVE {
            zbail!(ZCodecError::Invalid)
        }

        let has_ext = imsg::has_flag(header, flag::Z);
        if has_ext {
            extension::skip_all("Unknown KeepAlive ext", reader)?;
        }

        Ok(KeepAlive)
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        Self
    }
}
