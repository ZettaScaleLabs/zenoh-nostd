use zenoh_buffers::{
    reader::Reader,
    writer::Writer,
    zslice::{ZSlice, ZSliceLen},
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080, Zenoh080Bounded};

// ZSlice - Bounded
macro_rules! zslice_impl {
    ($bound:ty) => {
        impl<W> WCodec<&ZSlice, &mut W> for Zenoh080Bounded<$bound>
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &ZSlice) -> Self::Output {
                self.write(&mut *writer, x.len())?;
                writer.write_zslice(x)?;
                Ok(())
            }
        }

        impl<R, const L: usize> RCodec<ZSlice, &mut R> for (Zenoh080Bounded<$bound>, ZSliceLen<L>)
        where
            R: Reader,
        {
            type Error = ZError;

            #[allow(clippy::uninit_vec)]
            fn read(self, reader: &mut R) -> ZResult<ZSlice> {
                let len: usize = self.0.read(&mut *reader)?;
                if len > L {
                    bail!(ZE::CapacityExceeded);
                }

                let zslice = reader.read_zslice::<L>(len)?;
                Ok(zslice)
            }
        }
    };
}

zslice_impl!(u8);
zslice_impl!(u16);
zslice_impl!(u32);
zslice_impl!(u64);
zslice_impl!(usize);

// ZSlice
impl<W> WCodec<&ZSlice, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &ZSlice) -> Self::Output {
        let zodec = Zenoh080Bounded::<usize>::new();
        zodec.write(&mut *writer, x)
    }
}

impl<R, const L: usize> RCodec<ZSlice, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<ZSlice> {
        let zodec = (Zenoh080Bounded::<usize>::new(), ZSliceLen::<L>);
        zodec.read(&mut *reader)
    }
}
