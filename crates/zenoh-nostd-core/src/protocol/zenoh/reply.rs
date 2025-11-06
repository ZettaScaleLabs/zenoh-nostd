#[cfg(test)]
use crate::ZWriter;
#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{
    ZStruct,
    zenoh::{PushBody, query::ConsolidationMode},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|C|ID:5=0x4")]
pub struct Reply<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C))]
    pub consolidation: Option<ConsolidationMode>,

    // --- Body ---
    #[zenoh(size = remain)]
    pub payload: PushBody<'a>,
}

impl<'a> Reply<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let payload = PushBody::rand(w);
        let consolidation = thread_rng()
            .gen_bool(0.5)
            .then_some(ConsolidationMode::rand());

        Self {
            consolidation,
            payload,
        }
    }
}
