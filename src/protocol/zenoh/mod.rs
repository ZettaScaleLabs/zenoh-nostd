use crate::{
    protocol::{
        ZCodecError,
        common::imsg,
        zcodec::decode_u8,
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

pub(crate) mod id {
    pub(crate) const PUT: u8 = 0x01;
    pub(crate) const QUERY: u8 = 0x03;
    pub(crate) const REPLY: u8 = 0x04;
    pub(crate) const ERR: u8 = 0x05;
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

        match imsg::mid(header) {
            id::PUT => Ok(PushBody::Put(Put::decode(header, reader)?)),
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

        match imsg::mid(header) {
            id::QUERY => Ok(RequestBody::Query(Query::decode(header, reader)?)),
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

        match imsg::mid(header) {
            id::REPLY => Ok(ResponseBody::Reply(Reply::decode(header, reader)?)),
            id::ERR => Ok(ResponseBody::Err(Err::decode(header, reader)?)),
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

pub(crate) mod ext {
    use crate::{
        protocol::{
            ZCodecError,
            common::extension::ZExtZBufHeader,
            core::{EntityGlobalIdProto, ZenohIdProto, encoding::Encoding},
            zcodec::{
                decode_u8, decode_u32, decode_zbuf, encode_u8, encode_u32, encode_zbuf,
                encoded_len_u32, encoded_len_zbuf,
            },
        },
        result::ZResult,
        zbuf::{BufReaderExt, ZBufReader, ZBufWriter},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct SourceInfoType<const ID: u8> {
        pub(crate) id: EntityGlobalIdProto,
        pub(crate) sn: u32,
    }

    impl<const ID: u8> SourceInfoType<{ ID }> {
        pub(crate) fn encoded_len(&self) -> usize {
            1 + self.id.zid.encoded_len(false)
                + encoded_len_u32(self.id.eid)
                + encoded_len_u32(self.sn)
        }

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.encoded_len());
            header.encode(more, writer)?;

            let flags: u8 = (self.id.zid.size() as u8 - 1) << 4;
            encode_u8(writer, flags)?;

            self.id.zid.encode(false, writer)?;
            encode_u32(writer, self.id.eid)?;
            encode_u32(writer, self.sn)?;

            Ok(())
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (_, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;
            let flags = decode_u8(reader)?;
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
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id = EntityGlobalIdProto::rand();
            let sn: u32 = rng.r#gen();
            Self { id, sn }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct ValueType<'a, const VID: u8, const SID: u8> {
        pub(crate) encoding: Encoding<'a>,
        pub(crate) payload: crate::zbuf::ZBuf<'a>,
    }

    impl<'a, const VID: u8, const SID: u8> ValueType<'a, { VID }, { SID }> {
        pub(crate) const VID: u8 = VID;
        pub(crate) const SID: u8 = SID;

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ VID }> = ZExtZBufHeader::new(
                self.encoding.encoded_len() + encoded_len_zbuf(false, self.payload),
            );

            header.encode(more, writer)?;
            self.encoding.encode(writer)?;

            encode_zbuf(writer, false, self.payload)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (h, more) = ZExtZBufHeader::<{ VID }>::decode(header, reader)?;

            let start = reader.remaining();
            let encoding = Encoding::decode(reader)?;
            let end = reader.remaining();

            let len = h.len - (start - end);
            let payload = decode_zbuf(reader, Some(len))?;

            Ok((Self { encoding, payload }, more))
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
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
    pub(crate) struct AttachmentType<'a, const ID: u8> {
        pub(crate) buffer: crate::zbuf::ZBuf<'a>,
    }

    impl<'a, const ID: u8> AttachmentType<'a, { ID }> {
        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ ID }> =
                ZExtZBufHeader::new(encoded_len_zbuf(false, self.buffer));
            header.encode(more, writer)?;
            encode_zbuf(writer, false, self.buffer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (h, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;
            let buffer = decode_zbuf(reader, Some(h.len))?;
            Ok((Self { buffer }, more))
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
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
