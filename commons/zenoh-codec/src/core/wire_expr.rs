use zenoh_protocol::{
    core::wire_expr::{ExprId, WireExpr},
    network::Mapping,
};

use crate::{RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, WireExpr<'_>> for Zenoh080 {
    fn write(
        &self,
        message: WireExpr<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let WireExpr {
            scope,
            suffix,
            mapping: _,
        } = message;

        self.write(scope, writer)?;
        if !suffix.is_empty() {
            self.write(suffix, writer)?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, WireExpr<'a>> for (Zenoh080, bool) {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<WireExpr<'a>> {
        let scope: ExprId = self.0.read(reader)?;

        let suffix: &str = if self.1 { self.0.read(reader)? } else { "" };

        Ok(WireExpr {
            scope,
            suffix,
            mapping: Mapping::DEFAULT,
        })
    }
}
