use ryu_derive::ZStruct;

#[cfg(test)]
use crate::ByteWriter;
use crate::{
    marker,
    zenoh::{PushBody, query::ConsolidationMode},
};

#[derive(ZStruct, Debug, PartialEq)]
pub struct Reply<'a> {
    // --- Header ---
    _header: marker::Header,
    #[hstore(value = Self::ID)]
    _id: marker::Phantom,

    // --- Optional attributes ---
    #[option(header = Self::FLAG_C)]
    pub consolidation: Option<ConsolidationMode>,

    // --- Body ---
    #[size(deduced)]
    pub payload: PushBody<'a>,
}

impl Reply<'_> {
    pub const ID: u8 = 0b0000_0100;

    const FLAG_C: u8 = 0b0010_0000;
    const FLAG_Z: u8 = 0b1000_0000;
}

impl<'a> Reply<'a> {
    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> Self {
        use rand::{Rng, thread_rng};

        let mut rng = thread_rng();
        let payload = PushBody::rand(zbuf);
        let consolidation = rng.gen_bool(0.5).then_some(ConsolidationMode::rand());

        Self {
            _header: marker::Header,
            _id: marker::Phantom,

            consolidation,
            payload,
        }
    }
}
