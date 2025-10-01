use heapless::Vec;
use zenoh_buffer::{ZBuf, ZBufMut, ZBufReader, ZBufWriter};
use zenoh_protocol::{
    common::imsg,
    core::wire_expr::{ExprLen, WireExpr},
    network::{declare::common, Mapping},
};
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080};

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
