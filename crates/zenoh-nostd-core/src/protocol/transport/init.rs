use core::u16;

use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZReaderExt, ZStruct, ZWriter, ZWriterExt, ZenohIdProto,
    resolution::Resolution,
    transport::{Auth, HasCompression, HasLowLatency, HasQoS, MultiLink, PatchType, QoSLink},
    whatami::WhatAmI,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "ZID:4|_:2|W:2")]
pub struct InitIdentifier {
    #[zenoh(header = W)]
    pub whatami: WhatAmI,
    #[zenoh(size = header(ZID))]
    pub zid: ZenohIdProto,
}

#[derive(Debug, PartialEq)]
pub struct BatchSize(pub u16);

impl ZBodyLen for BatchSize {
    fn z_body_len(&self) -> usize {
        self.0.to_le_bytes().len()
    }
}

impl ZBodyEncode for BatchSize {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZCodecResult<()> {
        w.write(&self.0.to_le_bytes())?;
        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for BatchSize {
    type Ctx = ();

    fn z_body_decode(r: &mut crate::ZReader<'_>, _: ()) -> crate::ZCodecResult<Self> {
        let mut bytes = u16::MAX.to_le_bytes();
        r.read_into(&mut bytes)?;
        Ok(BatchSize(u16::from_le_bytes(bytes)))
    }
}

crate::__internal_zstructimpl!(BatchSize);

#[derive(ZStruct, Debug, PartialEq)]
pub struct InitResolution {
    pub resolution: Resolution,
    pub batch_size: BatchSize,
}

impl InitResolution {
    pub const DEFAULT: Self = Self {
        resolution: Resolution::DEFAULT,
        batch_size: BatchSize(u16::MAX),
    };
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|S|A|ID:5=0x01")]
pub struct InitSyn<'a> {
    pub version: u8,
    pub identifier: InitIdentifier,

    #[zenoh(presence = header(S), default = InitResolution::DEFAULT)]
    pub resolution: InitResolution,

    #[zenoh(ext = 0x1)]
    pub qos: Option<HasQoS>,
    #[zenoh(ext = 0x1)]
    pub qos_link: Option<QoSLink>,
    #[zenoh(ext = 0x2)]
    pub auth: Option<Auth<'a>>,
    #[zenoh(ext = 0x3)]
    pub mlink: Option<MultiLink<'a>>,
    #[zenoh(ext = 0x4)]
    pub lowlatency: Option<HasLowLatency>,
    #[zenoh(ext = 0x5)]
    pub compression: Option<HasCompression>,
    #[zenoh(ext = 0x6, default = PatchType::NONE)]
    pub patch: PatchType,
}

// #[derive(ZStruct, Debug, PartialEq)]
// pub struct InitSyn {
//     pub version: u8,
//     pub whatami: WhatAmI,
//     pub zid: ZenohIdProto,
//     pub resolution: Resolution,
//     pub batch_size: BatchSize,
//     pub ext_qos: Option<ext::QoS>,
//     pub ext_qos_link: Option<ext::QoSLink>,
//     #[cfg(feature = "shared-memory")]
//     pub ext_shm: Option<ext::Shm>,
//     pub ext_auth: Option<ext::Auth>,
//     pub ext_mlink: Option<ext::MultiLink>,
//     pub ext_lowlatency: Option<ext::LowLatency>,
//     pub ext_compression: Option<ext::Compression>,
//     pub ext_patch: ext::PatchType,
// }
