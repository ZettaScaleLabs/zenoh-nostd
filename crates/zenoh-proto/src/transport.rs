use core::time::Duration;

use crate::{
    NetworkBatch, Reliability, Resolution, WhatAmI, ZBodyDecode, ZBodyEncode, ZBodyLen, ZExt,
    ZReaderExt, ZStruct, ZWriter, ZWriterExt, ZenohIdProto, network::*,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:2|S|ID:5=0x03")]
pub struct Close {
    pub reason: u8,

    #[zenoh(header = S)]
    pub behaviour: CloseBehaviour,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|R|ID:5=0x05")]
pub struct FrameHeader {
    #[zenoh(header = R)]
    pub reliability: Reliability,
    pub sn: u32,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
}

#[derive(Debug, PartialEq)]
pub struct Frame<'a, 'b> {
    pub header: FrameHeader,
    pub msgs: NetworkBatch<'a, 'b>,
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        for _ in self.msgs.by_ref() {}
    }
}

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

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:3|ID:5=0x04")]
pub struct KeepAlive;

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:7")]
pub struct OpenExt<'a> {
    #[zenoh(ext = 0x1)]
    pub qos: Option<HasQoS>,
    #[zenoh(ext = 0x3)]
    pub auth: Option<Auth<'a>>,
    #[zenoh(ext = 0x5)]
    pub lowlatency: Option<HasLowLatency>,
    #[zenoh(ext = 0x6)]
    pub compression: Option<HasCompression>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|T|A:1=0|ID:5=0x02")]
pub struct OpenSyn<'a> {
    #[zenoh(flatten, shift = 6)]
    pub lease: Duration,
    pub sn: u32,

    #[zenoh(size = prefixed)]
    pub cookie: &'a [u8],

    #[zenoh(flatten)]
    pub ext: OpenExt<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|T|A:1=1|ID:5=0x02")]
pub struct OpenAck<'a> {
    #[zenoh(flatten, shift = 6)]
    pub lease: Duration,
    pub sn: u32,

    #[zenoh(flatten)]
    pub ext: OpenExt<'a>,
}

impl OpenExt<'_> {
    pub const DEFAULT: Self = Self {
        qos: None,
        auth: None,
        lowlatency: None,
        compression: None,
    };
}

#[derive(Debug, PartialEq)]
pub enum TransportBody<'a, 'b> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    Close(Close),
    KeepAlive(KeepAlive),
    Frame(Frame<'a, 'b>),
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum CloseBehaviour {
    #[default]
    Link = 0,
    Session = 1,
}

impl From<CloseBehaviour> for u8 {
    fn from(value: CloseBehaviour) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for CloseBehaviour {
    type Error = crate::ZCodecError;

    fn try_from(value: u8) -> crate::ZResult<Self, crate::ZCodecError> {
        match value {
            0 => Ok(CloseBehaviour::Link),
            1 => Ok(CloseBehaviour::Session),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BatchSize(pub u16);

impl ZBodyLen for BatchSize {
    fn z_body_len(&self) -> usize {
        self.0.to_le_bytes().len()
    }
}

impl ZBodyEncode for BatchSize {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        w.write(&self.0.to_le_bytes())?;
        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for BatchSize {
    type Ctx = ();

    fn z_body_decode(
        r: &mut crate::ZReader<'_>,
        _: (),
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let mut bytes = u16::MAX.to_le_bytes();
        r.read_into(&mut bytes)?;
        Ok(BatchSize(u16::from_le_bytes(bytes)))
    }
}

crate::derive_zstruct_with_body!(BatchSize);

#[derive(ZExt, Debug, PartialEq)]
pub struct HasQoS {}

#[derive(ZExt, Debug, PartialEq)]
pub struct Auth<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

#[derive(ZExt, Debug, PartialEq)]
pub struct MultiLink<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasLowLatency {}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasCompression {}

#[derive(ZExt, Debug, PartialEq)]
pub struct Patch {
    pub int: u8,
}

impl Patch {
    pub const NONE: Self = Self { int: 0 };
    pub const CURRENT: Self = Self { int: 1 };
}
