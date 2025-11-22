#[cfg(test)]
use rand::Rng;

use crate::{
    Resolution, WhatAmI, ZBodyDecode, ZBodyEncode, ZBodyLen, ZReaderExt, ZStruct, ZWriter,
    ZWriterExt, ZenohIdProto,
    transport::{Auth, HasCompression, HasLowLatency, HasQoS, MultiLink, Patch},
};

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

impl InitIdentifier {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter) -> Self {
        let whatami = WhatAmI::rand();
        let zid = ZenohIdProto::rand(w);
        Self { whatami, zid }
    }
}

impl InitResolution {
    pub const DEFAULT: Self = Self {
        resolution: Resolution::DEFAULT,
        batch_size: BatchSize(u16::MAX),
    };

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let resolution = Resolution::rand();
        let batch_size = BatchSize(rand::thread_rng().r#gen());
        Self {
            resolution,
            batch_size,
        }
    }
}

impl<'a> InitExt<'a> {
    pub const DEFAULT: Self = Self {
        qos: None,
        auth: None,
        mlink: None,
        lowlatency: None,
        compression: None,
        patch: Patch::CURRENT,
    };

    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let qos = if rand::thread_rng().gen_bool(0.5) {
            Some(HasQoS {})
        } else {
            None
        };
        let auth = if rand::thread_rng().gen_bool(0.5) {
            Some(Auth::rand(w))
        } else {
            None
        };
        let mlink = if rand::thread_rng().gen_bool(0.5) {
            Some(MultiLink::rand(w))
        } else {
            None
        };
        let lowlatency = if rand::thread_rng().gen_bool(0.5) {
            Some(HasLowLatency {})
        } else {
            None
        };
        let compression = if rand::thread_rng().gen_bool(0.5) {
            Some(HasCompression {})
        } else {
            None
        };
        let patch = if rand::thread_rng().gen_bool(0.5) {
            Patch::rand(w)
        } else {
            Patch::NONE
        };

        Self {
            qos,
            auth,
            mlink,
            lowlatency,
            compression,
            patch,
        }
    }
}

impl<'a> InitSyn<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let version = rand::thread_rng().r#gen();
        let identifier = InitIdentifier::rand(w);
        let resolution = if rand::thread_rng().gen_bool(0.5) {
            InitResolution::rand(w)
        } else {
            InitResolution::DEFAULT
        };
        let ext = InitExt::rand(w);

        Self {
            version,
            identifier,
            resolution,
            ext,
        }
    }
}

impl<'a> InitAck<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let version = rand::thread_rng().r#gen();
        let identifier = InitIdentifier::rand(w);
        let resolution = if rand::thread_rng().gen_bool(0.5) {
            InitResolution::rand(w)
        } else {
            InitResolution::DEFAULT
        };
        let cookie_len = rand::thread_rng().gen_range(0..16);
        let cookie = w
            .write_slot(cookie_len, |b: &mut [u8]| {
                rand::thread_rng().fill(b);
                b.len()
            })
            .unwrap();
        let ext = InitExt::rand(w);

        Self {
            version,
            identifier,
            resolution,
            cookie,
            ext,
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

crate::derive_zstruct_with_body!(BatchSize);
