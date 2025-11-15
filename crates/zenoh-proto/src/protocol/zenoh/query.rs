#[cfg(test)]
use {
    crate::{ZWriter, ZWriterExt},
    rand::{
        Rng,
        distributions::{Alphanumeric, DistString},
        thread_rng,
    },
};

use crate::{
    ZStruct,
    zenoh::{Attachment, ConsolidationMode, SourceInfo, Value},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|P|C|ID:5=0x3")]
pub struct Query<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C), default = ConsolidationMode::default())]
    pub consolidation: ConsolidationMode,
    #[zenoh(presence = header(P), size = prefixed, default = "")]
    pub parameters: &'a str,

    // --- Extension Block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,
    #[zenoh(ext = 0x3)]
    pub body: Option<Value<'a>>,
    #[zenoh(ext = 0x5)]
    pub attachment: Option<Attachment<'a>>,
}

impl<'a> Query<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        const MIN: usize = 1;
        const MAX: usize = 16;

        let consolidation = if thread_rng().gen_bool(0.5) {
            ConsolidationMode::rand()
        } else {
            ConsolidationMode::default()
        };

        let parameters = if thread_rng().gen_bool(0.5) {
            let len = thread_rng().gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut thread_rng(), len);
            w.write_str(proto.as_str()).unwrap()
        } else {
            ""
        };
        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let body = thread_rng().gen_bool(0.5).then_some(Value::rand(w));
        let attachment = thread_rng().gen_bool(0.5).then_some(Attachment::rand(w));

        Self {
            consolidation,
            parameters,

            sinfo,
            body,
            attachment,
        }
    }
}
