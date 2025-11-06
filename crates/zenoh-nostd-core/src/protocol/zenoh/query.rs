#[cfg(test)]
use crate::ZWriterExt;
#[cfg(test)]
use rand::{
    Rng,
    distributions::{Alphanumeric, DistString},
    prelude::SliceRandom,
    thread_rng,
};

use crate::{
    ZCodecError, ZCodecResult, ZReader, ZStruct, ZStructDecode, ZStructEncode, ZWriter,
    zenoh::{Attachment, SourceInfo, Value},
};

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Copy)]
pub enum ConsolidationMode {
    #[default]
    Auto = 0,
    None = 1,
    Monotonic = 2,
    Latest = 3,
}

impl ConsolidationMode {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        *[Self::None, Self::Monotonic, Self::Latest, Self::Auto]
            .choose(&mut thread_rng())
            .unwrap()
    }
}

impl ZStructEncode for ConsolidationMode {
    fn z_len(&self) -> usize {
        <u64 as ZStructEncode>::z_len(&((*self as u8) as u64))
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZStructEncode>::z_encode(&((*self as u8) as u64), w)
    }
}
impl<'a> ZStructDecode<'a> for ConsolidationMode {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        let value = <u64 as ZStructDecode>::z_decode(r)?;
        match value as u8 {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|P|C|ID:5=0x3")]
pub struct Query<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C))]
    pub consolidation: Option<ConsolidationMode>,
    #[zenoh(presence = header(P), size = prefixed)]
    pub parameters: Option<&'a str>,

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

        let consolidation = thread_rng()
            .gen_bool(0.5)
            .then_some(ConsolidationMode::rand());
        let parameters = if thread_rng().gen_bool(0.5) {
            let len = thread_rng().gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut thread_rng(), len);
            Some(w.write_str(proto.as_str()).unwrap())
        } else {
            None
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
