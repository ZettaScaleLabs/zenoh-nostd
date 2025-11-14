#[cfg(test)]
use crate::{ZWriter, ZWriterExt};
#[cfg(test)]
use rand::{Rng, thread_rng};

use uhlc::Timestamp;

use crate::{
    ZStruct,
    encoding::Encoding,
    zenoh::{Attachment, SourceInfo},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|E|T|ID:5=0x1")]
pub struct Put<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(T))]
    pub timestamp: Option<Timestamp>,
    #[zenoh(presence = header(E), default = Encoding::EMPTY)]
    pub encoding: Encoding<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,
    #[zenoh(ext = 0x3)]
    pub attachment: Option<Attachment<'a>>,

    // --- Body ---
    #[zenoh(size = prefixed)]
    pub payload: &'a [u8],
}

impl<'a> Put<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let timestamp = thread_rng().gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let encoding = if thread_rng().gen_bool(0.5) {
            Encoding::rand(w)
        } else {
            Encoding::EMPTY
        };

        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let attachment = thread_rng().gen_bool(0.5).then_some(Attachment::rand(w));
        let payload = w
            .write_slot(thread_rng().gen_range(1..=64), |b| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self {
            timestamp,
            encoding,
            sinfo,
            attachment,
            payload,
        }
    }
}
