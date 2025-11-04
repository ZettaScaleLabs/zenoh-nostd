use ryu_derive::ZStruct;

#[cfg(test)]
use crate::ByteWriter;
use crate::{encoding::Encoding, marker, zenoh::SourceInfo, zextfield};

#[derive(ZStruct, Debug, PartialEq)]
pub struct Err<'a> {
    // --- Header definition ---
    _header: marker::Header,

    #[hstore(value = Self::ID)]
    _id: marker::Phantom,

    // --- Optional attributes ---
    #[option(header = Self::FLAG_E)]
    pub encoding: Option<Encoding<'a>>,

    // --- Extension block ---
    #[option(header = Self::FLAG_Z)]
    _ebegin: marker::ExtBlockBegin,
    pub sinfo: Option<SourceInfo>,
    _eend: marker::ExtBlockEnd,

    // --- Body ---
    #[size(deduced)]
    pub payload: &'a [u8],
}

zextfield!(impl<'a> SourceInfo, Err<'a>, 0x1, false);

impl Err<'_> {
    pub const ID: u8 = 0b0000_0101;

    const FLAG_E: u8 = 0b0100_0000;
    const FLAG_Z: u8 = 0b1000_0000;
}

impl<'a> Err<'a> {
    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> Self {
        use crate::ByteWriterExt;
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let encoding = rng.gen_bool(0.5).then_some(Encoding::rand(zbuf));
        let sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let payload = zbuf
            .write_slot(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();

        Self {
            _header: marker::Header,
            _id: marker::Phantom,
            encoding,

            _ebegin: marker::ExtBlockBegin,
            sinfo,
            _eend: marker::ExtBlockEnd,

            payload,
        }
    }
}
