mod encoding;
mod locator;
mod timestamp;
mod wire_expr;
mod zbuf;
mod zenohid;
mod zint;
mod zslice;

use zenoh_buffers::{reader::Reader, writer::Writer};
use zenoh_result::{zerr, ZError, ZResult, ZE};

use heapless::{String, Vec};

use crate::{LCodec, RCodec, WCodec, Zenoh080, Zenoh080Bounded};

// [u8; N]
macro_rules! array_impl {
    ($n:expr) => {
        impl<W> WCodec<[u8; $n], &mut W> for Zenoh080
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: [u8; $n]) -> Self::Output {
                writer.write_exact(x.as_slice())
            }
        }

        impl<W> WCodec<&[u8; $n], &mut W> for Zenoh080
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &[u8; $n]) -> Self::Output {
                self.write(writer, *x)
            }
        }

        impl<R> RCodec<[u8; $n], &mut R> for Zenoh080
        where
            R: Reader,
        {
            type Error = ZError;

            fn read(self, reader: &mut R) -> ZResult<[u8; $n]> {
                let mut x = [0u8; $n];
                reader.read_exact(&mut x)?;
                Ok(x)
            }
        }

        impl LCodec<[u8; $n]> for Zenoh080 {
            fn w_len(self, _: [u8; $n]) -> usize {
                $n
            }
        }
    };
}

array_impl!(1);
array_impl!(2);
array_impl!(3);
array_impl!(4);
array_impl!(5);
array_impl!(6);
array_impl!(7);
array_impl!(8);
array_impl!(9);
array_impl!(10);
array_impl!(11);
array_impl!(12);
array_impl!(13);
array_impl!(14);
array_impl!(15);
array_impl!(16);

// &[u8] / Vec<u8> - Bounded
macro_rules! vec_impl {
    ($bound:ty) => {
        impl<W> WCodec<&[u8], &mut W> for Zenoh080Bounded<$bound>
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &[u8]) -> Self::Output {
                self.write(&mut *writer, x.len())?;
                if x.is_empty() {
                    Ok(())
                } else {
                    writer.write_exact(x)
                }
            }
        }

        impl<R, const N: usize> RCodec<Vec<u8, N>, &mut R> for Zenoh080Bounded<$bound>
        where
            R: Reader,
        {
            type Error = ZError;

            #[allow(clippy::uninit_vec)]
            fn read(self, reader: &mut R) -> ZResult<Vec<u8, N>> {
                let len: usize = self.read(&mut *reader)?;
                let mut buff = zenoh_buffers::vec::uninit::<N>();
                if len != 0 {
                    reader.read_exact(&mut buff[..])?;
                }
                Ok(buff)
            }
        }
    };
}

vec_impl!(u8);
vec_impl!(u16);
vec_impl!(u32);
vec_impl!(u64);
vec_impl!(usize);

// &[u8] / Vec<u8>
impl LCodec<&[u8]> for Zenoh080 {
    fn w_len(self, x: &[u8]) -> usize {
        self.w_len(x.len()) + x.len()
    }
}

impl<W> WCodec<&[u8], &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &[u8]) -> Self::Output {
        let zcodec = Zenoh080Bounded::<usize>::new();
        zcodec.write(&mut *writer, x)
    }
}

impl<R, const N: usize> RCodec<Vec<u8, N>, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Vec<u8, N>> {
        let zcodec = Zenoh080Bounded::<usize>::new();
        zcodec.read(&mut *reader)
    }
}

// &str / String - Bounded
macro_rules! str_impl {
    ($bound:ty) => {
        impl<W> WCodec<&str, &mut W> for Zenoh080Bounded<$bound>
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &str) -> Self::Output {
                self.write(&mut *writer, x.as_bytes())
            }
        }

        impl<W, const N: usize> WCodec<&String<N>, &mut W> for Zenoh080Bounded<$bound>
        where
            W: Writer,
        {
            type Output = ZResult<()>;

            fn write(self, writer: &mut W, x: &String<N>) -> Self::Output {
                self.write(&mut *writer, x.as_str())
            }
        }

        impl<R, const N: usize> RCodec<String<N>, &mut R> for Zenoh080Bounded<$bound>
        where
            R: Reader,
        {
            type Error = ZError;

            #[allow(clippy::uninit_vec)]
            fn read(self, reader: &mut R) -> ZResult<String<N>> {
                let vec: Vec<u8, N> = self.read(&mut *reader)?;
                String::from_utf8(vec).map_err(|_| zerr!(ZE::DidntRead))
            }
        }
    };
}

str_impl!(u8);
str_impl!(u16);
str_impl!(u32);
str_impl!(u64);
str_impl!(usize);

// &str / String
impl LCodec<&str> for Zenoh080 {
    fn w_len(self, x: &str) -> usize {
        self.w_len(x.as_bytes())
    }
}

impl<const N: usize> LCodec<&String<N>> for Zenoh080 {
    fn w_len(self, x: &String<N>) -> usize {
        self.w_len(x.as_bytes())
    }
}

impl<W> WCodec<&str, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &str) -> Self::Output {
        let zcodec = Zenoh080Bounded::<usize>::new();
        zcodec.write(&mut *writer, x)
    }
}

impl<W, const N: usize> WCodec<&String<N>, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &String<N>) -> Self::Output {
        let zcodec = Zenoh080Bounded::<usize>::new();
        zcodec.write(&mut *writer, x)
    }
}

impl<R, const N: usize> RCodec<String<N>, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<String<N>> {
        let zcodec = Zenoh080Bounded::<usize>::new();
        zcodec.read(&mut *reader)
    }
}
