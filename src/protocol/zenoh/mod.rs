use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_u8, decode_u64, encode_u64},
        msg_id,
        zenoh::{err::Err, put::Put, query::Query, reply::Reply},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod err;
pub(crate) mod put;
pub(crate) mod query;
pub(crate) mod reply;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PushBody<'a> {
    Put(Put<'a>),
}

impl<'a> PushBody<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            PushBody::Put(put) => put.encode(writer),
        }
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = decode_u8(reader)?;

        match msg_id(header) {
            Put::ID => Ok(PushBody::Put(Put::decode(reader, header)?)),
            _ => zbail!(ZCodecError::CouldNotRead),
        }
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..1) {
            0 => PushBody::Put(Put::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RequestBody<'a> {
    Query(Query<'a>),
}

impl<'a> RequestBody<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            RequestBody::Query(query) => query.encode(writer),
        }
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = decode_u8(reader)?;

        match msg_id(header) {
            Query::ID => Ok(RequestBody::Query(Query::decode(reader, header)?)),
            _ => zbail!(ZCodecError::CouldNotRead),
        }
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..1) {
            0 => RequestBody::Query(Query::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResponseBody<'a> {
    Reply(Reply<'a>),
    Err(Err<'a>),
}

impl<'a> ResponseBody<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            ResponseBody::Reply(reply) => reply.encode(writer),
            ResponseBody::Err(err) => err.encode(writer),
        }
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = decode_u8(reader)?;

        match msg_id(header) {
            Reply::ID => Ok(ResponseBody::Reply(Reply::decode(reader, header)?)),
            Err::ID => Ok(ResponseBody::Err(Err::decode(reader, header)?)),
            _ => {
                zbail!(ZCodecError::CouldNotRead)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            0 => ResponseBody::Reply(Reply::rand(zbuf)),
            1 => ResponseBody::Err(Err::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub(crate) enum ConsolidationMode {
    #[default]
    Auto,
    None,
    Monotonic,
    Latest,
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

pub(crate) fn encode_consolidation_mode(
    writer: &mut ZBufWriter<'_>,
    consolidation: &ConsolidationMode,
) -> ZResult<(), ZCodecError> {
    let x: u64 = match consolidation {
        ConsolidationMode::Auto => 0,
        ConsolidationMode::None => 1,
        ConsolidationMode::Monotonic => 2,
        ConsolidationMode::Latest => 3,
    };

    encode_u64(writer, x)?;

    Ok(())
}

pub(crate) fn decode_consolidation_mode(
    reader: &mut ZBufReader<'_>,
) -> ZResult<ConsolidationMode, ZCodecError> {
    let x = decode_u64(reader)?;

    match x {
        0 => Ok(ConsolidationMode::Auto),
        1 => Ok(ConsolidationMode::None),
        2 => Ok(ConsolidationMode::Monotonic),
        3 => Ok(ConsolidationMode::Latest),
        _ => Ok(ConsolidationMode::Auto),
    }
}
