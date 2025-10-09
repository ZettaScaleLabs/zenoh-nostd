use crate::{
    protocol::{
        ZCodecError,
        common::imsg,
        zenoh::{err::Err, put::Put, query::Query, reply::Reply},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

pub mod id {
    pub const OAM: u8 = 0x00;
    pub const PUT: u8 = 0x01;
    pub const DEL: u8 = 0x02;
    pub const QUERY: u8 = 0x03;
    pub const REPLY: u8 = 0x04;
    pub const ERR: u8 = 0x05;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushBody<'a> {
    Put(Put<'a>),
}

impl<'a> PushBody<'a> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            PushBody::Put(put) => put.encode(writer),
        }
    }

    pub fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = crate::protocol::zcodec::decode_u8(reader)?;

        match imsg::mid(header) {
            id::PUT => Ok(PushBody::Put(Put::decode(header, reader)?)),
            _ => zbail!(ZCodecError::Invalid),
        }
    }

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..1) {
            0 => PushBody::Put(Put::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody<'a> {
    Query(Query<'a>),
}

impl<'a> RequestBody<'a> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            RequestBody::Query(query) => query.encode(writer),
        }
    }

    pub fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = crate::protocol::zcodec::decode_u8(reader)?;

        match imsg::mid(header) {
            id::QUERY => Ok(RequestBody::Query(Query::decode(header, reader)?)),
            _ => zbail!(ZCodecError::Invalid),
        }
    }

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..1) {
            0 => RequestBody::Query(Query::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseBody<'a> {
    Reply(Reply<'a>),
    Err(Err<'a>),
}

impl<'a> ResponseBody<'a> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            ResponseBody::Reply(reply) => reply.encode(writer),
            ResponseBody::Err(err) => err.encode(writer),
        }
    }

    pub fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header = crate::protocol::zcodec::decode_u8(reader)?;

        match imsg::mid(header) {
            id::REPLY => Ok(ResponseBody::Reply(Reply::decode(header, reader)?)),
            id::ERR => Ok(ResponseBody::Err(Err::decode(header, reader)?)),
            _ => {
                zbail!(ZCodecError::Invalid)
            }
        }
    }

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            0 => ResponseBody::Reply(Reply::rand(zbuf)),
            1 => ResponseBody::Err(Err::rand(zbuf)),
            _ => unreachable!(),
        }
    }
}

pub mod ext {
    use crate::{
        protocol::{
            ZCodecError,
            common::extension::ZExtZBufHeader,
            core::{EntityGlobalIdProto, ZenohIdProto, encoding::Encoding},
            zcodec::{
                decode_u32, decode_zbuf, encode_u8, encode_u32, encode_zbuf, encoded_len_u32,
                encoded_len_zbuf,
            },
        },
        result::ZResult,
        zbuf::{BufReaderExt, ZBufReader, ZBufWriter},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SourceInfoType<const ID: u8> {
        pub id: EntityGlobalIdProto,
        pub sn: u32,
    }

    impl<const ID: u8> SourceInfoType<{ ID }> {
        pub fn encoded_len(&self) -> usize {
            1 + self.id.zid.encoded_len(false)
                + encoded_len_u32(self.id.eid)
                + encoded_len_u32(self.sn)
        }

        pub fn encode(&self, more: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.encoded_len());
            header.encode(more, writer)?;

            let flags: u8 = (self.id.zid.size() as u8 - 1) << 4;
            encode_u8(flags, writer)?;

            self.id.zid.encode(false, writer)?;
            encode_u32(self.id.eid, writer)?;
            encode_u32(self.sn, writer)?;

            Ok(())
        }

        pub fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (_, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;
            let flags = crate::protocol::zcodec::decode_u8(reader)?;
            let length = 1 + ((flags >> 4) as usize);

            let zid = ZenohIdProto::decode(Some(length), reader)?;
            let eid = decode_u32(reader)?;
            let sn = decode_u32(reader)?;

            Ok((
                Self {
                    id: EntityGlobalIdProto { zid, eid },
                    sn,
                },
                more,
            ))
        }

        #[cfg(test)]
        pub fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id = EntityGlobalIdProto::rand();
            let sn: u32 = rng.r#gen();
            Self { id, sn }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ValueType<'a, const VID: u8, const SID: u8> {
        pub encoding: Encoding<'a>,
        pub payload: crate::zbuf::ZBuf<'a>,
    }

    impl<'a, const VID: u8, const SID: u8> ValueType<'a, { VID }, { SID }> {
        pub const VID: u8 = VID;
        pub const SID: u8 = SID;

        pub fn encode(&self, more: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ VID }> = ZExtZBufHeader::new(
                self.encoding.encoded_len() + encoded_len_zbuf(false, self.payload),
            );

            header.encode(more, writer)?;
            self.encoding.encode(writer)?;

            encode_zbuf(false, self.payload, writer)
        }

        pub fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (h, more) = ZExtZBufHeader::<{ VID }>::decode(header, reader)?;

            let start = reader.remaining();
            let encoding = Encoding::decode(reader)?;
            let end = reader.remaining();

            let len = h.len - (start - end);
            let payload = decode_zbuf(Some(len), reader)?;

            Ok((Self { encoding, payload }, more))
        }

        #[cfg(test)]
        pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;

            use crate::zbuf::BufWriterExt;
            let mut rng = rand::thread_rng();

            let encoding = Encoding::rand(zbuf);
            let payload = zbuf
                .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                    rng.fill(b);
                    b.len()
                })
                .unwrap();

            Self { encoding, payload }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AttachmentType<'a, const ID: u8> {
        pub buffer: crate::zbuf::ZBuf<'a>,
    }

    impl<'a, const ID: u8> AttachmentType<'a, { ID }> {
        pub fn encode(&self, more: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ ID }> =
                ZExtZBufHeader::new(encoded_len_zbuf(false, self.buffer));
            header.encode(more, writer)?;
            encode_zbuf(false, self.buffer, writer)
        }

        pub fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (h, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;
            let buffer = decode_zbuf(Some(h.len), reader)?;
            Ok((Self { buffer }, more))
        }

        #[cfg(test)]
        pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;

            use crate::zbuf::BufWriterExt;
            let mut rng = rand::thread_rng();
            let buffer = zbuf
                .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                    rng.fill(b);
                    b.len()
                })
                .unwrap();

            Self { buffer }
        }
    }
}
