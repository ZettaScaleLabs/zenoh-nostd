#[cfg(test)]
use {
    crate::ZWriter,
    rand::{Rng, thread_rng},
};

use crate::{
    ZStruct,
    zenoh::{ConsolidationMode, PushBody},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|C|ID:5=0x4")]
pub struct Reply<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C), default = ConsolidationMode::default())]
    pub consolidation: ConsolidationMode,

    // --- Body ---
    pub payload: PushBody<'a>,
}

impl<'a> Reply<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let payload = PushBody::rand(w);

        let consolidation = if thread_rng().gen_bool(0.5) {
            ConsolidationMode::rand()
        } else {
            ConsolidationMode::default()
        };

        Self {
            consolidation,
            payload,
        }
    }
}
