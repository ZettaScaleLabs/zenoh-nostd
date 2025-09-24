use core::convert::TryFrom;

use heapless::{String, Vec};
use zenoh_buffers::{reader::Reader, writer::Writer};
use zenoh_protocol::core::Locator;
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080, Zenoh080Bounded};

impl<W, const N: usize> WCodec<&Locator<N>, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &Locator<N>) -> Self::Output {
        let zodec = Zenoh080Bounded::<u8>::new();
        zodec.write(writer, x.as_str())
    }
}

impl<R, const N: usize> RCodec<Locator<N>, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Locator<N>> {
        let zodec = Zenoh080Bounded::<u8>::new();
        let loc: String<N> = zodec.read(reader)?;
        Locator::try_from(loc).map_err(|_| zerr!(ZE::CapacityExceeded))
    }
}

impl<W, const N: usize> WCodec<&[Locator<N>], &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &[Locator<N>]) -> Self::Output {
        self.write(&mut *writer, x.len())?;
        for l in x {
            self.write(&mut *writer, l)?;
        }
        Ok(())
    }
}

impl<R, const N: usize, const L: usize> RCodec<Vec<Locator<L>, N>, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Vec<Locator<L>, N>> {
        let len: usize = self.read(&mut *reader)?;
        if len > N {
            bail!(ZE::CapacityExceeded);
        }

        let mut vec: Vec<Locator<L>, N> = Vec::new();
        for _ in 0..len {
            vec.push(self.read(&mut *reader)?)
                .map_err(|_| zerr!(ZE::CapacityExceeded))?;
        }
        Ok(vec)
    }
}
