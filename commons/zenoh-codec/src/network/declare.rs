use heapless::Vec;
use zenoh_buffer::{ZBuf, ZBufMut, ZBufReader, ZBufWriter};
use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::wire_expr::{ExprId, ExprLen, WireExpr},
    network::{
        declare::{self, common, keyexpr, queryable, subscriber, token, Declare, DeclareBody},
        id, Mapping,
    },
};
use zenoh_result::{zbail, zctx, zerr, WithContext, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

// Declaration
impl<'a> WCodec<'a, &DeclareBody<'_>> for Zenoh080 {
    fn write(self, message: &DeclareBody, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        match message {
            DeclareBody::DeclareKeyExpr(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::UndeclareKeyExpr(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::DeclareSubscriber(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::UndeclareSubscriber(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::DeclareQueryable(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::UndeclareQueryable(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::DeclareToken(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::UndeclareToken(r) => self.write(r, writer).ctx(zctx!())?,
            DeclareBody::DeclareFinal(r) => self.write(r, writer).ctx(zctx!())?,
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, DeclareBody<'a>> for Zenoh080 {
    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<DeclareBody<'a>> {
        let header: u8 = self.read(reader)?;

        let d = match imsg::mid(header) {
            declare::id::D_KEYEXPR => {
                DeclareBody::DeclareKeyExpr(self.read_knowing_header(reader, header).ctx(zctx!())?)
            }
            declare::id::U_KEYEXPR => DeclareBody::UndeclareKeyExpr(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            ),
            declare::id::D_SUBSCRIBER => DeclareBody::DeclareSubscriber(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            ),
            declare::id::U_SUBSCRIBER => DeclareBody::UndeclareSubscriber(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            ),
            declare::id::D_QUERYABLE => DeclareBody::DeclareQueryable(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            ),
            declare::id::U_QUERYABLE => DeclareBody::UndeclareQueryable(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            ),
            declare::id::D_TOKEN => {
                DeclareBody::DeclareToken(self.read_knowing_header(reader, header).ctx(zctx!())?)
            }
            declare::id::U_TOKEN => {
                DeclareBody::UndeclareToken(self.read_knowing_header(reader, header).ctx(zctx!())?)
            }
            declare::id::D_FINAL => {
                DeclareBody::DeclareFinal(self.read_knowing_header(reader, header).ctx(zctx!())?)
            }
            _ => zbail!(ZE::ReadFailure),
        };

        Ok(d)
    }
}

// Declare
impl<'a> WCodec<'a, &Declare<'_>> for Zenoh080 {
    fn write(self, message: &Declare<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let Declare {
            interest_id,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            body,
        } = message;

        // Header
        let mut header = id::DECLARE;

        if interest_id.is_some() {
            header |= declare::flag::I;
        }

        let mut n_exts = ((ext_qos != &declare::ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + ((ext_nodeid != &declare::ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= declare::flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;

        if let Some(interest_id) = interest_id {
            self.write(*interest_id, writer).ctx(zctx!())?;
        }

        if ext_qos != &declare::ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_qos, n_exts != 0), writer).ctx(zctx!())?;
        }
        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write((ts, n_exts != 0), writer).ctx(zctx!())?;
        }
        if ext_nodeid != &declare::ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_nodeid, n_exts != 0), writer)
                .ctx(zctx!())?;
        }

        self.write(body, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a> RCodec<'a, Declare<'a>> for Zenoh080 {
    fn read_knowing_header(&self, reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Declare<'a>> {
        if imsg::mid(header) != id::DECLARE {
            zbail!(ZE::ReadFailure);
        }

        let mut interest_id = None;
        if imsg::has_flag(header, declare::flag::I) {
            interest_id = Some(self.read(reader)?);
        }

        // Extensions
        let mut ext_qos = declare::ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = declare::ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, declare::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
            match iext::eid(ext) {
                declare::ext::QoS::ID => {
                    let (q, ext): (declare::ext::QoSType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos = q;
                    has_ext = ext;
                }
                declare::ext::Timestamp::ID => {
                    let (t, ext): (declare::ext::TimestampType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                declare::ext::NodeId::ID => {
                    let (nid, ext): (declare::ext::NodeIdType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Declare", ext)?;
                }
            }
        }

        let body: DeclareBody = self.read(reader)?;

        Ok(Declare {
            interest_id,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            body,
        })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<Declare<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

// Final
impl<'a> WCodec<'a, &common::DeclareFinal> for Zenoh080 {
    fn write(self, message: &common::DeclareFinal, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let header = declare::id::D_FINAL;
        self.write(header, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, common::DeclareFinal> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<common::DeclareFinal> {
        if imsg::mid(header) != declare::id::D_FINAL {
            zbail!(ZE::ReadFailure);
        }

        let has_ext = imsg::has_flag(header, token::flag::Z);
        if has_ext {
            extension::skip_all(reader, "Final")?;
        }

        Ok(common::DeclareFinal)
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<common::DeclareFinal> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}

impl<'a> WCodec<'a, &keyexpr::DeclareKeyExpr<'_>> for Zenoh080 {
    fn write(self, message: &keyexpr::DeclareKeyExpr, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let keyexpr::DeclareKeyExpr { id, wire_expr } = message;

        let mut header = declare::id::D_KEYEXPR;
        if wire_expr.has_suffix() {
            header |= keyexpr::flag::N;
        }
        self.write(header, writer).ctx(zctx!())?;
        self.write(*id, writer).ctx(zctx!())?;

        self.write(wire_expr, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, keyexpr::DeclareKeyExpr<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<keyexpr::DeclareKeyExpr<'a>> {
        if imsg::mid(header) != declare::id::D_KEYEXPR {
            zbail!(ZE::ReadFailure);
        }

        let id: ExprId = self.read(reader)?;
        let wire_expr: WireExpr<'_> = self
            .read_with_condition(reader, imsg::has_flag(header, keyexpr::flag::N))
            .ctx(zctx!())?;

        let has_ext = imsg::has_flag(header, keyexpr::flag::Z);
        if has_ext {
            extension::skip_all(reader, "DeclareKeyExpr")?;
        }

        Ok(keyexpr::DeclareKeyExpr { id, wire_expr })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<keyexpr::DeclareKeyExpr<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

// UndeclareKeyExpr
impl<'a> WCodec<'a, &keyexpr::UndeclareKeyExpr> for Zenoh080 {
    fn write(
        self,
        message: &keyexpr::UndeclareKeyExpr,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let keyexpr::UndeclareKeyExpr { id } = message;

        let header = declare::id::U_KEYEXPR;
        self.write(header, writer).ctx(zctx!())?;

        self.write(*id, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, keyexpr::UndeclareKeyExpr> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<keyexpr::UndeclareKeyExpr> {
        if imsg::mid(header) != declare::id::U_KEYEXPR {
            zbail!(ZE::ReadFailure);
        }

        let id: ExprId = self.read(reader)?;

        // Extensions
        let has_ext = imsg::has_flag(header, keyexpr::flag::Z);
        if has_ext {
            extension::skip_all(reader, "UndeclareKeyExpr")?;
        }

        Ok(keyexpr::UndeclareKeyExpr { id })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<keyexpr::UndeclareKeyExpr> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &subscriber::DeclareSubscriber<'_>> for Zenoh080 {
    fn write(
        self,
        message: &subscriber::DeclareSubscriber,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let subscriber::DeclareSubscriber { id, wire_expr } = message;

        let mut header = declare::id::D_SUBSCRIBER;
        if wire_expr.mapping != Mapping::DEFAULT {
            header |= subscriber::flag::M;
        }
        if wire_expr.has_suffix() {
            header |= subscriber::flag::N;
        }
        self.write(header, writer).ctx(zctx!())?;
        self.write(*id, writer).ctx(zctx!())?;

        self.write(wire_expr, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, subscriber::DeclareSubscriber<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<subscriber::DeclareSubscriber<'a>> {
        if imsg::mid(header) != declare::id::D_SUBSCRIBER {
            zbail!(ZE::ReadFailure);
        }

        let id: subscriber::SubscriberId = self.read(reader)?;
        let mut wire_expr: WireExpr<'_> = self
            .read_with_condition(reader, imsg::has_flag(header, subscriber::flag::N))
            .ctx(zctx!())?;
        wire_expr.mapping = if imsg::has_flag(header, subscriber::flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        let mut has_ext = imsg::has_flag(header, subscriber::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
            has_ext = extension::skip(reader, "DeclareSubscriber", ext)?;
        }

        Ok(subscriber::DeclareSubscriber { id, wire_expr })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<subscriber::DeclareSubscriber<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &subscriber::UndeclareSubscriber<'_>> for Zenoh080 {
    fn write(
        self,
        message: &subscriber::UndeclareSubscriber<'_>,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let subscriber::UndeclareSubscriber { id, ext_wire_expr } = message;

        let mut header = declare::id::U_SUBSCRIBER;
        if !ext_wire_expr.is_null() {
            header |= subscriber::flag::Z;
        }
        self.write(header, writer).ctx(zctx!())?;
        self.write(*id, writer).ctx(zctx!())?;

        if !ext_wire_expr.is_null() {
            self.write((ext_wire_expr, false), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, subscriber::UndeclareSubscriber<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<subscriber::UndeclareSubscriber<'a>> {
        if imsg::mid(header) != declare::id::U_SUBSCRIBER {
            zbail!(ZE::ReadFailure);
        }

        let id: subscriber::SubscriberId = self.read(reader)?;

        let mut ext_wire_expr = common::ext::WireExprType::null();

        let mut has_ext = imsg::has_flag(header, subscriber::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
            match iext::eid(ext) {
                common::ext::WireExprExt::ID => {
                    let (we, ext): (common::ext::WireExprType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_wire_expr = we;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "UndeclareSubscriber", ext)?;
                }
            }
        }

        Ok(subscriber::UndeclareSubscriber { id, ext_wire_expr })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<subscriber::UndeclareSubscriber<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

// QueryableInfo
impl<'a> WCodec<'a, (&queryable::ext::QueryableInfoType, bool)> for Zenoh080 {
    fn write(
        self,
        message: (&queryable::ext::QueryableInfoType, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;

        let mut flags: u8 = 0;
        if x.complete {
            flags |= queryable::ext::flag::C;
        }
        let v: u64 = (flags as u64) | ((x.distance as u64) << 8);
        let ext = queryable::ext::QueryableInfo::new(v);

        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, (queryable::ext::QueryableInfoType, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(queryable::ext::QueryableInfoType, bool)> {
        let (ext, more): (queryable::ext::QueryableInfo, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let complete = imsg::has_flag(ext.value as u8, queryable::ext::flag::C);
        let distance = (ext.value >> 8) as u16;

        Ok((
            queryable::ext::QueryableInfoType { complete, distance },
            more,
        ))
    }
}

impl<'a> WCodec<'a, &queryable::DeclareQueryable<'_>> for Zenoh080 {
    fn write(
        self,
        message: &queryable::DeclareQueryable<'_>,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let queryable::DeclareQueryable {
            id,
            wire_expr,
            ext_info,
        } = message;

        let mut header = declare::id::D_QUERYABLE;
        let mut n_exts = (ext_info != &queryable::ext::QueryableInfoType::DEFAULT) as u8;

        if n_exts != 0 {
            header |= subscriber::flag::Z;
        }

        if wire_expr.mapping != Mapping::DEFAULT {
            header |= subscriber::flag::M;
        }

        if wire_expr.has_suffix() {
            header |= subscriber::flag::N;
        }

        self.write(header, writer).ctx(zctx!())?;

        // Body
        self.write(*id, writer).ctx(zctx!())?;
        self.write(wire_expr, writer).ctx(zctx!())?;

        if ext_info != &queryable::ext::QueryableInfoType::DEFAULT {
            n_exts -= 1;
            self.write((ext_info, n_exts != 0), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, queryable::DeclareQueryable<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<queryable::DeclareQueryable<'a>> {
        if imsg::mid(header) != declare::id::D_QUERYABLE {
            zbail!(ZE::ReadFailure);
        }

        // Body
        let id: queryable::QueryableId = self.read(reader)?;
        let mut wire_expr: WireExpr<'_> = self
            .read_with_condition(reader, imsg::has_flag(header, queryable::flag::N))
            .ctx(zctx!())?;

        wire_expr.mapping = if imsg::has_flag(header, queryable::flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        let mut ext_info = queryable::ext::QueryableInfoType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, queryable::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
            match iext::eid(ext) {
                queryable::ext::QueryableInfo::ID => {
                    let (i, ext): (queryable::ext::QueryableInfoType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_info = i;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "DeclareQueryable", ext)?;
                }
            }
        }

        Ok(queryable::DeclareQueryable {
            id,
            wire_expr,
            ext_info,
        })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<queryable::DeclareQueryable<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &queryable::UndeclareQueryable<'_>> for Zenoh080 {
    fn write(
        self,
        message: &queryable::UndeclareQueryable,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let queryable::UndeclareQueryable { id, ext_wire_expr } = message;

        let header = declare::id::U_QUERYABLE | queryable::flag::Z;
        self.write(header, writer).ctx(zctx!())?;
        self.write(*id, writer).ctx(zctx!())?;

        self.write((ext_wire_expr, false), writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, queryable::UndeclareQueryable<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<queryable::UndeclareQueryable<'a>> {
        if imsg::mid(header) != declare::id::U_QUERYABLE {
            zbail!(ZE::ReadFailure);
        }

        // Body
        let id: queryable::QueryableId = self.read(reader).ctx(zctx!())?;

        // Extensions
        let mut ext_wire_expr = common::ext::WireExprType::null();

        let mut has_ext = imsg::has_flag(header, queryable::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
            match iext::eid(ext) {
                common::ext::WireExprExt::ID => {
                    let (we, ext): (common::ext::WireExprType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_wire_expr = we;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "UndeclareQueryable", ext)?;
                }
            }
        }

        Ok(queryable::UndeclareQueryable { id, ext_wire_expr })
    }

    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<queryable::UndeclareQueryable<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &token::DeclareToken> for Zenoh080 {
    fn write(self, message: &token::DeclareToken, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let token::DeclareToken { id, wire_expr } = message;

        // Header
        let mut header = declare::id::D_TOKEN;
        if wire_expr.mapping != Mapping::DEFAULT {
            header |= subscriber::flag::M;
        }
        if wire_expr.has_suffix() {
            header |= subscriber::flag::N;
        }
        self.write(writer, header)?;

        // Body
        self.write(writer, id)?;
        self.write(writer, wire_expr)?;

        Ok(())
    }
}

impl<'a> RCodec<'a, token::DeclareToken> for Zenoh080 {
    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<token::DeclareToken> {
        let header: u8 = self.read(reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<'a> RCodec<'a, token::DeclareToken> for Zenoh080Header {
    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<token::DeclareToken> {
        if imsg::mid(self.header) != declare::id::D_TOKEN {
            zbail!(ZE::ReadFailure);
        }

        // Body
        let id: token::TokenId = self.codec.read(reader)?;
        let ccond = Zenoh080Condition::new(imsg::has_flag(self.header, token::flag::N));
        let mut wire_expr: WireExpr<'static> = ccond.read(reader)?;
        wire_expr.mapping = if imsg::has_flag(self.header, token::flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        // Extensions
        let has_ext = imsg::has_flag(self.header, token::flag::Z);
        if has_ext {
            extension::skip_all(reader, "DeclareToken")?;
        }

        Ok(token::DeclareToken { id, wire_expr })
    }
}

// UndeclareToken
impl<'a> WCodec<'a, &token::UndeclareToken> for Zenoh080 {
    fn write(self, message: &token::UndeclareToken, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let token::UndeclareToken { id, ext_wire_expr } = message;

        // Header
        let header = declare::id::U_TOKEN | token::flag::Z;
        self.write(writer, header)?;

        // Body
        self.write(writer, id)?;

        // Extension
        self.write(writer, (ext_wire_expr, false))?;

        Ok(())
    }
}

impl<'a> RCodec<'a, token::UndeclareToken> for Zenoh080 {
    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<token::UndeclareToken> {
        let header: u8 = self.read(reader)?;
        let codec = Zenoh080Header::new(header);

        codec.read(reader)
    }
}

impl<'a> RCodec<'a, token::UndeclareToken> for Zenoh080Header {
    fn read(self, reader: &mut ZBufReader<'a>) -> ZResult<token::UndeclareToken> {
        if imsg::mid(self.header) != declare::id::U_TOKEN {
            zbail!(ZE::ReadFailure);
        }

        // Body
        let id: token::TokenId = self.codec.read(reader)?;

        // Extensions
        let mut ext_wire_expr = common::ext::WireExprType::null();

        let mut has_ext = imsg::has_flag(self.header, token::flag::Z);
        while has_ext {
            let ext: u8 = self.codec.read(reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                common::ext::WireExprExt::ID => {
                    let (we, ext): (common::ext::WireExprType, bool) = eodec.read(reader)?;
                    ext_wire_expr = we;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "UndeclareToken", ext)?;
                }
            }
        }

        Ok(token::UndeclareToken { id, ext_wire_expr })
    }
}

// LLIMIT

impl<'a> WCodec<'a, (&common::ext::WireExprType<'_>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&common::ext::WireExprType, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let common::ext::WireExprType { wire_expr } = x;

        let mut data = [0u8; 256]; // Temporary buffer for the inner encoding, assume max 256 bytes
        let mut value = ZBufMut(&mut data);
        let mut zriter = value.writer();

        let mut flags: u8 = 0;

        if x.wire_expr.has_suffix() {
            flags |= 1;
        }

        if let Mapping::Sender = wire_expr.mapping {
            flags |= 1 << 1;
        }

        self.write(flags, &mut zriter).ctx(zctx!())?;
        self.write(wire_expr.scope, &mut zriter).ctx(zctx!())?;

        if wire_expr.has_suffix() {
            zriter
                .write_exact(wire_expr.suffix.as_bytes())
                .ctx(zctx!())?;
        }

        let zbuf_len = zriter.pos();
        let value = value.into_ref();

        let ext = common::ext::WireExprExt {
            value: value.subslice(0..zbuf_len)?,
        };

        self.write((&ext, more), writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a> RCodec<'a, (common::ext::WireExprType<'a>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(common::ext::WireExprType<'a>, bool)> {
        let (ext, more): (common::ext::WireExprExt<'a>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let mut zeader = ext.value.local_reader();
        let flags: u8 = self.read(&mut zeader)?;

        let scope: ExprLen = self.read(&mut zeader)?;
        let suffix: &str = if imsg::has_flag(flags, 1) {
            let len = zeader.remaining();
            let zbuf = zeader.read_zbuf(len).ctx(zctx!())?;

            zbuf.as_str().ctx(zctx!())?
        } else {
            ""
        };
        let mapping = if imsg::has_flag(flags, 1 << 1) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        Ok((
            common::ext::WireExprType {
                wire_expr: WireExpr {
                    scope,
                    suffix: suffix.into(),
                    mapping,
                },
            },
            more,
        ))
    }
}
