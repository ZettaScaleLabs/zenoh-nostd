use crate::{
    ByteIOError, ByteIOResult, ByteWriter, ZStruct, marker,
    zenoh::{Attachment, SourceInfo, Value},
    zextfield,
};

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
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
        use rand::prelude::SliceRandom;
        let mut rng = rand::thread_rng();

        *[Self::None, Self::Monotonic, Self::Latest, Self::Auto]
            .choose(&mut rng)
            .unwrap()
    }
}

impl ZStruct for ConsolidationMode {
    fn z_len(&self) -> usize {
        <u64 as ZStruct>::z_len(&((*self as u8) as u64))
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        <u64 as ZStruct>::z_encode(&((*self as u8) as u64), w)
    }

    type ZType<'a> = ConsolidationMode;
    fn z_decode<'a>(r: &mut crate::ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let value = <u64 as ZStruct>::z_decode(r)?;
        match value as u8 {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Err(ByteIOError::CouldNotParse),
        }
    }
}

#[derive(ZStruct, Debug, PartialEq)]
pub struct Query<'a> {
    // --- Header ---
    _header: marker::Header,
    #[hstore(value = Self::ID)]
    _id: marker::Phantom,

    // --- Optional attributes ---
    #[option(header = Self::FLAG_C)]
    pub consolidation: Option<ConsolidationMode>,
    #[option(header = Self::FLAG_P, size(plain))]
    pub parameters: Option<&'a str>,

    // --- Extension Block ---
    #[option(header = Self::FLAG_Z)]
    _ebegin: marker::ExtBlockBegin,
    pub sinfo: Option<SourceInfo>,
    pub body: Option<Value<'a>>,
    pub attachment: Option<Attachment<'a>>,
    _eend: marker::ExtBlockEnd,
}

zextfield!(impl<'a> SourceInfo, Query<'a>, 0x1, false);
zextfield!(impl<'a> Attachment<'a>, Query<'a>, 0x3, false);
zextfield!(impl<'a> Value<'a>, Query<'a>, 0x5, false);

impl Query<'_> {
    pub const ID: u8 = 0b0000_0011;

    const FLAG_C: u8 = 0b0010_0000;
    const FLAG_P: u8 = 0b0100_0000;
    const FLAG_Z: u8 = 0b1000_0000;
}

impl<'a> Query<'a> {
    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> Self {
        use rand::{
            Rng,
            distributions::{Alphanumeric, DistString},
        };

        let mut rng = rand::thread_rng();

        const MIN: usize = 1;
        const MAX: usize = 16;

        let consolidation = rng.gen_bool(0.5).then_some(ConsolidationMode::rand());
        let parameters = if rng.gen_bool(0.5) {
            use crate::ByteWriterExt;

            let len = rng.gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut rng, len);
            Some(zbuf.write_str(proto.as_str()).unwrap())
        } else {
            None
        };
        let sinfo = rng.gen_bool(0.5).then_some(SourceInfo::rand());
        let body = rng.gen_bool(0.5).then_some(Value::rand(zbuf));
        let attachment = rng.gen_bool(0.5).then_some(Attachment::rand(zbuf));

        Self {
            _header: marker::Header,
            _id: marker::Phantom,

            consolidation,
            parameters,

            _ebegin: marker::ExtBlockBegin,
            sinfo,
            body,
            attachment,
            _eend: marker::ExtBlockEnd,
        }
    }
}
