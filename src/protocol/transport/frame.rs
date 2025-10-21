#[cfg(test)]
use heapless::Vec;

use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::Reliability,
        network::NetworkMessage,
        transport::{TransportSn, id},
        zcodec::{decode_u8, decode_u32, encode_u8, encode_u32},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const R: u8 = 1 << 5;

    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame<'a, 'b> {
    pub(crate) reliability: Reliability,
    pub(crate) sn: TransportSn,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) payload: &'b [NetworkMessage<'a>],
}

impl<'a, 'b> Frame<'a, 'b> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let header = FrameHeader {
            reliability: self.reliability,
            sn: self.sn,
            ext_qos: self.ext_qos,
        };

        header.encode(writer)?;

        for msg in self.payload {
            msg.encode(writer)?;
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn rand(
        zbuf: &mut ZBufWriter<'a>,
        vec: &'b mut Vec<NetworkMessage<'a>, 16>,
    ) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let reliability = Reliability::DEFAULT;
        let sn: TransportSn = rng.r#gen();
        let ext_qos = ext::QoSType::rand();

        vec.clear();
        let len = rng.gen_range(1..16);
        let payload = {
            for _ in 0..len {
                vec.push(NetworkMessage::rand(zbuf)).unwrap();
            }
            vec.as_slice()
        };

        Frame {
            reliability,
            sn,
            ext_qos,
            payload,
        }
    }
}

pub(crate) mod ext {
    pub(crate) type QoS = crate::zextz64!(0x1, true);
    pub(crate) type QoSType = crate::protocol::transport::ext::QoSType<{ QoS::ID }>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct FrameHeader {
    pub(crate) reliability: Reliability,
    pub(crate) sn: TransportSn,
    pub(crate) ext_qos: ext::QoSType,
}

impl FrameHeader {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::FRAME;

        if let Reliability::Reliable = self.reliability {
            header |= flag::R;
        }

        if self.ext_qos != ext::QoSType::DEFAULT {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;
        encode_u32(writer, self.sn)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            self.ext_qos.encode(false, writer)?;
        }

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::FRAME {
            zbail!(ZCodecError::CouldNotRead)
        }

        let reliability = match imsg::has_flag(header, flag::R) {
            true => Reliability::Reliable,
            false => Reliability::BestEffort,
        };
        let sn: TransportSn = decode_u32(reader)?;

        let mut ext_qos = ext::QoSType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoSType::decode(ext, reader)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Frame", ext, reader)?;
                }
            }
        }

        Ok(FrameHeader {
            reliability,
            sn,
            ext_qos,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let reliability = Reliability::rand();
        let sn: TransportSn = rng.r#gen();
        let ext_qos = ext::QoSType::rand();

        FrameHeader {
            reliability,
            sn,
            ext_qos,
        }
    }
}
