#[cfg(test)]
use {
    crate::{ZWriter, ZWriterExt},
    rand::{Rng, thread_rng},
};

use crate::{Encoding, ZStruct, zenoh::SourceInfo};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|E|_|ID:5=0x5")]
pub struct Err<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(E), default = Encoding::DEFAULT)]
    pub encoding: Encoding<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,

    // --- Body ---
    #[zenoh(size = prefixed)]
    pub payload: &'a [u8],
}

impl<'a> Err<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let encoding = if thread_rng().gen_bool(0.5) {
            Encoding::rand(w)
        } else {
            Encoding::DEFAULT
        };

        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self {
            encoding,
            sinfo,
            payload,
        }
    }
}
