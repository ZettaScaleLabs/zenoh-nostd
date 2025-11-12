use crate::{
    Reliability, ZStruct,
    network::{NetworkBodyIter, QoS},
};

#[cfg(test)]
use rand::Rng;

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x05")]
pub struct FrameHeader {
    pub reliability: Reliability,
    pub sn: u32,

    pub qos: QoS,
}

impl Frame<'_, '_> {
    pub const ID: u8 = FrameHeader::ID;
}

#[derive(Debug, PartialEq)]
pub struct Frame<'a, 'b> {
    pub header: FrameHeader,
    pub msgs: NetworkBodyIter<'a, 'b>,
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        for _ in self.msgs.by_ref() {}
    }
}

impl FrameHeader {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let reliability = Reliability::rand(w);
        let sn = rand::thread_rng().r#gen();
        let qos = QoS::rand(w);
        Self {
            reliability,
            sn,
            qos,
        }
    }
}

// #[derive(ZEncode, Debug, PartialEq)]
// #[zenoh(header = "Z|_:2|ID:5=0x05")]
// pub struct Frame<'a> {
//     #[zenoh(flatten)]
//     pub frame: FrameHeader,

//     #[zenoh(size = remain)]
//     pub payload: &'a [u8],
// }

// impl<'a, 'b> Frame<'a, 'b> {
//     const HEADER_BASE: u8 = 5u8 << 0u8;
//     const HEADER_SLOT_Z: u8 = 0b1 << 7u8;

//     pub const ID: u8 = 5u8;
// }

// impl crate::ZHeader for Frame<'_, '_> {
//     fn z_header(&self) -> u8 {
//         let Self { .. } = self;
//         let mut header: u8 = Self::HEADER_BASE;
//         header |= if <_ as crate::ZExtCount>::z_ext_count(self) > 0 {
//             Self::HEADER_SLOT_Z
//         } else {
//             0
//         };
//         header
//     }
// }

// impl crate::ZExtCount for Frame<'_, '_> {
//     fn z_ext_count(&self) -> usize {
//         let mut n_exts = 0;
//         let Self { qos, .. } = self;
//         if qos != &QoS::DEFAULT {
//             let _ = qos;
//             n_exts += 1;
//         }
//         n_exts
//     }
// }

// impl crate::ZBodyLen for Frame<'_, '_> {
//     fn z_body_len(&self) -> usize {
//         let Self {
//             reliability,
//             sn,
//             qos,
//             ..
//         } = self;
//         let mut l = 0
//             + <_ as crate::ZLen>::z_len(reliability)
//             + <_ as crate::ZLen>::z_len(sn)
//             + if qos != &QoS::DEFAULT {
//                 crate::zext_len::<_>(qos)
//             } else {
//                 0usize
//             };

//         for payload in self.payload.iter() {
//             l += <_ as crate::ZLen>::z_len(payload);
//         }

//         l
//     }
// }

// impl crate::ZLen for Frame<'_, '_> {
//     fn z_len(&self) -> usize {
//         1 + <_ as crate::ZBodyLen>::z_body_len(self)
//     }
// }

// impl crate::ZBodyEncode for Frame<'_, '_> {
//     fn z_body_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
//         let Self {
//             reliability,
//             sn,
//             qos,
//             ..
//         } = self;
//         <_ as crate::ZEncode>::z_encode(reliability, w)?;
//         <_ as crate::ZEncode>::z_encode(sn, w)?;
//         let mut n_exts = <_ as crate::ZExtCount>::z_ext_count(self);
//         if qos != &QoS::DEFAULT {
//             n_exts -= 1;
//             crate::zext_encode::<_, 0x1, true>(qos, w, n_exts != 0)?;
//         }
//         for payload in self.payload.iter() {
//             <_ as crate::ZEncode>::z_encode(payload, w)?;
//         }
//         Ok(())
//     }
// }

// impl crate::ZEncode for Frame<'_, '_> {
//     fn z_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
//         let h = <Self as crate::ZHeader>::z_header(self);
//         <u8 as crate::ZEncode>::z_encode(&h, w)?;
//         <_ as crate::ZBodyEncode>::z_body_encode(self, w)?;
//         Ok(())
//     }
// }

// impl<'a, 'b> crate::ZBodyDecode<'a> for Frame<'a, 'b> {
//     type Ctx = u8;

//     fn z_body_decode(r: &mut crate::ZReader<'a>, header: u8) -> crate::ZCodecResult<Self> {
//         let reliability = { <_ as crate::ZDecode>::z_decode(r)? };
//         let sn = { <_ as crate::ZDecode>::z_decode(r)? };
//         let mut has_ext: bool = header & Self::HEADER_SLOT_Z != 0;
//         let mut qos = QoS::DEFAULT;
//         while has_ext {
//             let (ext_id, ext_kind, mandatory, more) = crate::decode_ext_header(r)?;
//             has_ext = more;
//             match ext_id {
//                 0x1 => {
//                     qos = crate::zext_decode::<_>(r)?;
//                 }
//                 _ => {
//                     if mandatory {
//                         return Err(crate::ZCodecError::UnsupportedMandatoryExtension);
//                     }
//                     crate::skip_ext(r, ext_kind)?;
//                 }
//             }
//         }

//         Ok(Self {
//             reliability,
//             sn,
//             qos,
//         })
//     }
// }
// impl<'a, 'b> crate::ZDecode<'a> for Frame<'a, 'b> {
//     fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZCodecResult<Self> {
//         let h = <u8 as crate::ZDecode>::z_decode(r)?;
//         <_ as crate::ZBodyDecode>::z_body_decode(r, h)
//     }
// }
