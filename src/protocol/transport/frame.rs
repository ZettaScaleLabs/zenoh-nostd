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
        zcodec::{decode_u32, encode_u32},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub mod flag {
    pub const R: u8 = 1 << 5;

    pub const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<'a, 'b> {
    pub reliability: Reliability,
    pub sn: TransportSn,
    pub ext_qos: ext::QoSType,
    pub payload: &'b [NetworkMessage<'a>],
}

impl<'a, 'b> Frame<'a, 'b> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
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
    pub fn rand(zbuf: &mut ZBufWriter<'a>, vec: &'b mut Vec<NetworkMessage<'a>, 16>) -> Self {
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

pub mod ext {
    pub type QoS = crate::zextz64!(0x1, true);
    pub type QoSType = crate::protocol::transport::ext::QoSType<{ QoS::ID }>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub reliability: Reliability,
    pub sn: TransportSn,
    pub ext_qos: ext::QoSType,
}

impl FrameHeader {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::FRAME;

        if let Reliability::Reliable = self.reliability {
            header |= flag::R;
        }

        if self.ext_qos != ext::QoSType::DEFAULT {
            header |= flag::Z;
        }

        crate::protocol::zcodec::encode_u8(header, writer)?;
        encode_u32(self.sn, writer)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            self.ext_qos.encode(false, writer)?;
        }

        Ok(())
    }

    pub fn decode(header: u8, reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::FRAME {
            zbail!(ZCodecError::Invalid)
        }

        let reliability = match imsg::has_flag(header, flag::R) {
            true => Reliability::Reliable,
            false => Reliability::BestEffort,
        };
        let sn: TransportSn = decode_u32(reader)?;

        let mut ext_qos = ext::QoSType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = crate::protocol::zcodec::decode_u8(reader)?;
            match iext::eid(ext) {
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
    pub fn rand() -> Self {
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
