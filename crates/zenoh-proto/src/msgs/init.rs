use crate::{exts::*, fields::*, *};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "ZID:4|_:2|W:2")]
pub struct InitIdentifier {
    #[zenoh(header = W)]
    pub whatami: WhatAmI,
    #[zenoh(size = header(ZID))]
    pub zid: ZenohIdProto,
}

#[derive(ZStruct, Debug, PartialEq)]
pub struct InitResolution {
    pub resolution: Resolution,
    pub batch_size: BatchSize,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:7")]
pub struct InitExt<'a> {
    #[zenoh(ext = 0x1)]
    pub qos: Option<HasQoS>,
    // TODO: support this extension WITH A DIFFERENT ID
    // #[zenoh(ext = 0x1)]
    // pub qos_link: Option<QoSLink>,
    #[zenoh(ext = 0x3)]
    pub auth: Option<Auth<'a>>,
    #[zenoh(ext = 0x4)]
    pub mlink: Option<MultiLink<'a>>,
    #[zenoh(ext = 0x5)]
    pub lowlatency: Option<HasLowLatency>,
    #[zenoh(ext = 0x6)]
    pub compression: Option<HasCompression>,
    #[zenoh(ext = 0x7, default = Patch::NONE)]
    pub patch: Patch,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|S|A:1=0|ID:5=0x01")]
pub struct InitSyn<'a> {
    pub version: u8,
    pub identifier: InitIdentifier,

    #[zenoh(presence = header(S), default = InitResolution::DEFAULT)]
    pub resolution: InitResolution,

    #[zenoh(flatten)]
    pub ext: InitExt<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|S|A:1=1|ID:5=0x01")]
pub struct InitAck<'a> {
    pub version: u8,
    pub identifier: InitIdentifier,

    #[zenoh(presence = header(S), default = InitResolution::DEFAULT)]
    pub resolution: InitResolution,

    #[zenoh(size = prefixed)]
    pub cookie: &'a [u8],

    #[zenoh(flatten)]
    pub ext: InitExt<'a>,
}

impl InitResolution {
    pub const DEFAULT: Self = Self {
        resolution: Resolution::DEFAULT,
        batch_size: BatchSize(u16::MAX),
    };
}

impl InitExt<'_> {
    pub const DEFAULT: Self = Self {
        qos: None,
        auth: None,
        mlink: None,
        lowlatency: None,
        compression: None,
        patch: Patch::CURRENT,
    };
}
