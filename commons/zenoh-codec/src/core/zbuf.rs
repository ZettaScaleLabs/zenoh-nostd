use zenoh_buffers::{buffer::Buffer, reader::Reader, writer::Writer, zbuf::ZBuf};

use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{LCodec, RCodec, WCodec, Zenoh080, Zenoh080Bounded};

// ZBuf bounded
macro_rules! zbuf_impl {
    ($bound:ty) => {
        impl<const N: usize, const L: usize> LCodec<&ZBuf<N, L>> for Zenoh080Bounded<$bound> {
            fn w_len(self, message: &ZBuf<N, L>) -> usize {
                message.len()
            }
        }

        impl<W, const N: usize, const L: usize> WCodec<&ZBuf<N, L>, &mut W>
            for Zenoh080Bounded<$bound>
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &ZBuf<N, L>) -> Self::Output {
                self.write(&mut *writer, x.len())?;
                for s in x.zslices() {
                    writer.write_zslice(s)?;
                }
                Ok(())
            }
        }

        impl<R, const N: usize, const L: usize> RCodec<ZBuf<N, L>, &mut R>
            for Zenoh080Bounded<$bound>
        where
            R: Reader,
        {
            type Error = ZError;

            fn read(self, reader: &mut R) -> ZResult<ZBuf<N, L>> {
                let len: usize = self.read(&mut *reader)?;
                let mut zbuf = ZBuf::empty();
                if N < 1 {
                    bail!(ZE::CapacityExceeded);
                }
                if len > N {
                    bail!(ZE::CapacityExceeded);
                }
                reader
                    .read_zslices::<_, L>(len, |s| zbuf.push_zslice(s).expect("Capacity Error"))?;
                Ok(zbuf)
            }
        }
    };
}

zbuf_impl!(u8);
zbuf_impl!(u16);
zbuf_impl!(u32);
zbuf_impl!(u64);
zbuf_impl!(usize);

// ZBuf flat
impl<W, const N: usize, const L: usize> WCodec<&ZBuf<N, L>, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &ZBuf<N, L>) -> Self::Output {
        let zodec = Zenoh080Bounded::<usize>::new();
        zodec.write(&mut *writer, x)
    }
}

impl<R, const N: usize, const L: usize> RCodec<ZBuf<N, L>, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<ZBuf<N, L>> {
        let zodec = Zenoh080Bounded::<usize>::new();
        zodec.read(&mut *reader)
    }
}

impl<const N: usize, const L: usize> LCodec<&ZBuf<N, L>> for Zenoh080 {
    fn w_len(self, message: &ZBuf<N, L>) -> usize {
        let zodec = Zenoh080Bounded::<usize>::new();
        zodec.w_len(message)
    }
}
