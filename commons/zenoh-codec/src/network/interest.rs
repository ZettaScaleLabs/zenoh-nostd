use zenoh_protocol::{
    common::{
        extension::iext,
        imsg::{self, HEADER_BITS},
    },
    core::wire_expr::WireExpr,
    network::{
        declare, id,
        interest::{self, Interest, InterestMode, InterestOptions},
        Mapping,
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, &Interest<'_>> for ZCodec {
    fn write(
        &self,
        message: &Interest<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Interest {
            id,
            mode,
            options: _,
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        } = message;

        let mut header = id::INTEREST;
        header |= match mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << HEADER_BITS;

        let mut n_exts = ((ext_qos != &declare::ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + ((ext_nodeid != &declare::ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= declare::flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;
        self.write(*id, writer).ctx(zctx!())?;

        if *mode != InterestMode::Final {
            self.write(message.options(), writer).ctx(zctx!())?;
            if let Some(we) = wire_expr.as_ref() {
                self.write(we, writer).ctx(zctx!())?;
            }
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

        Ok(())
    }
}

impl<'a> RCodec<'a, Interest<'a>> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Interest<'a>> {
        if imsg::mid(header) != id::INTEREST {
            zbail!(ZE::ReadFailure);
        }

        let id = self.read(reader).ctx(zctx!())?;
        let mode = match (header >> HEADER_BITS) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(ZE::ReadFailure),
        };

        let mut options = InterestOptions::empty();
        let mut wire_expr = None;
        if mode != InterestMode::Final {
            let options_byte: u8 = self.read(reader).ctx(zctx!())?;
            options = InterestOptions::from(options_byte);
            if options.restricted() {
                let mut we: WireExpr<'_> = self
                    .read_with_condition(reader, options.named())
                    .ctx(zctx!())?;
                we.mapping = if options.mapping() {
                    Mapping::Sender
                } else {
                    Mapping::Receiver
                };
                wire_expr = Some(we);
            }
        }

        let mut ext_qos = declare::ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = declare::ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, declare::flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            match iext::eid(ext) {
                declare::ext::QoS::ID => {
                    let (q, ext): (interest::ext::QoSType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos = q;
                    has_ext = ext;
                }
                declare::ext::Timestamp::ID => {
                    let (t, ext): (interest::ext::TimestampType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                declare::ext::NodeId::ID => {
                    let (nid, ext): (interest::ext::NodeIdType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Declare", ext)?;
                }
            }
        }

        Ok(Interest {
            id,
            mode,
            options,
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Interest<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
