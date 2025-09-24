use heapless::String;
use zenoh_buffers::{reader::Reader, writer::Writer};
use zenoh_protocol::{
    core::{CowStr, ExprId, ExprLen, WireExpr},
    network::Mapping,
};
use zenoh_result::{ZError, ZResult};

use crate::{core::Zenoh080Bounded, RCodec, WCodec, Zenoh080, Zenoh080Condition};

impl<W, const N: usize> WCodec<&WireExpr<'_, N>, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &WireExpr<'_, N>) -> Self::Output {
        let WireExpr {
            scope,
            suffix,
            mapping: _,
        } = x;

        let zodec = Zenoh080Bounded::<ExprId>::new();
        zodec.write(&mut *writer, *scope)?;

        if x.has_suffix() {
            let zodec = Zenoh080Bounded::<ExprLen>::new();
            zodec.write(&mut *writer, suffix.as_ref())?;
        }
        Ok(())
    }
}

impl<R, const N: usize> RCodec<WireExpr<'static, N>, &mut R> for Zenoh080Condition
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<WireExpr<'static, N>> {
        let zodec = Zenoh080Bounded::<ExprId>::new();
        let scope: ExprId = zodec.read(&mut *reader)?;

        let suffix: String<N> = if self.condition {
            let zodec = Zenoh080Bounded::<ExprLen>::new();
            zodec.read(&mut *reader)?
        } else {
            String::new()
        };
        Ok(WireExpr {
            scope,
            suffix: CowStr::Owned(suffix),
            mapping: Mapping::DEFAULT,
        })
    }
}
