use ryu_derive::ZStruct;
use uhlc::Timestamp;

#[cfg(test)]
use crate::ByteWriter;
use crate::{
    encoding::Encoding,
    marker,
    zenoh::{Attachment, SourceInfo},
    zextfield,
};

#[derive(ZStruct, Debug, PartialEq)]
pub struct Put<'a> {
    // --- Header definition ---
    _header: marker::Header,

    #[hstore(value = Self::ID)]
    _id: marker::Phantom,

    // --- Optional attributes ---
    #[option(header = Self::FLAG_T)]
    pub timestamp: Option<Timestamp>,
    #[option(header = Self::FLAG_E)]
    pub encoding: Option<Encoding<'a>>,

    // --- Extension block ---
    #[option(header = Self::FLAG_Z)]
    _ebegin: marker::ExtBlockBegin,
    pub sinfo: Option<SourceInfo>,
    pub attachment: Option<Attachment<'a>>,
    _eend: marker::ExtBlockEnd,

    // --- Body ---
    #[size(deduced)]
    pub payload: &'a [u8],
}

zextfield!(impl<'a> SourceInfo, Put<'a>, 0x1, false);
zextfield!(impl<'a> Attachment<'a>, Put<'a>, 0x3, false);

impl Put<'_> {
    pub(crate) const ID: u8 = 0b0000_0001;

    pub(crate) const FLAG_T: u8 = 0b0010_0000;
    pub(crate) const FLAG_E: u8 = 0b0100_0000;
    pub(crate) const FLAG_Z: u8 = 0b1000_0000;
}

impl<'a> Put<'a> {
    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> Self {
        use rand::Rng;

        use crate::ByteWriterExt;

        let mut rng = rand::thread_rng();
        let timestamp = rng.gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(rng.r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });
        let encoding = rng.gen_bool(0.5).then_some(Encoding::rand(zbuf));
        let sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let attachment = rng.gen_bool(0.5).then_some(Attachment::rand(zbuf));
        let payload = zbuf
            .write_slot(rng.gen_range(1..=64), |b| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self {
            _header: marker::Header,
            _id: marker::Phantom,
            timestamp,
            encoding,
            _ebegin: marker::ExtBlockBegin,
            sinfo,
            attachment,
            _eend: marker::ExtBlockEnd,
            payload,
        }
    }
}
