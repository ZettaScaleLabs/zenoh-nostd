use crate::{
    ZStruct,
    transport::{Auth, HasCompression, HasLowLatency, HasQoS},
};
use core::time::Duration;

#[cfg(test)]
use {crate::ZWriterExt, rand::Rng};

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

impl<'a> OpenExt<'a> {
    pub const DEFAULT: Self = Self {
        qos: None,
        auth: None,
        lowlatency: None,
        compression: None,
    };

    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
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

        OpenExt {
            qos,
            auth,
            lowlatency,
            compression,
        }
    }
}

impl<'a> OpenSyn<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let lease = Duration::from_secs(rand::thread_rng().gen_range(1..=3600));
        let sn: u32 = rand::thread_rng().r#gen();
        let cookie = w
            .write_slot(rand::thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                rand::thread_rng().fill(b);
                b.len()
            })
            .unwrap();
        let ext = OpenExt::rand(w);

        Self {
            lease,
            sn,
            cookie,
            ext,
        }
    }
}

impl<'a> OpenAck<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let lease = Duration::from_secs(rand::thread_rng().gen_range(1..=3600));
        let sn: u32 = rand::thread_rng().r#gen();
        let ext = OpenExt::rand(w);

        Self { lease, sn, ext }
    }
}
