use zenoh_protocol::{
    core::wire_expr::{ExprId, WireExpr},
    network::Mapping,
};
use zenoh_result::{zctx, WithContext};

use crate::{RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, &WireExpr<'_>> for ZCodec {
    fn write(
        &self,
        message: &WireExpr<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let WireExpr {
            scope,
            suffix,
            mapping: _,
        } = message;

        self.write(*scope, writer).ctx(zctx!())?;
        if !suffix.is_empty() {
            self.write(*suffix, writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> WCodec<'a, WireExpr<'_>> for ZCodec {
    fn write(
        &self,
        message: WireExpr<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, WireExpr<'a>> for ZCodec {
    fn read_with_condition(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        condition: bool,
    ) -> zenoh_result::ZResult<WireExpr<'a>> {
        let scope: ExprId = self.read(reader).ctx(zctx!())?;

        let suffix: &str = if condition {
            self.read(reader).ctx(zctx!())?
        } else {
            ""
        };

        Ok(WireExpr {
            scope,
            suffix,
            mapping: Mapping::DEFAULT,
        })
    }
}
