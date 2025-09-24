#![no_std]

pub mod cow;
pub mod zbuf;
pub mod zslice;

mod slice;
pub mod vec;

#[macro_export]
macro_rules! unsafe_slice_mut {
    ($s:expr,$r:expr) => {{
        let slice = &mut *$s;
        let index = $r;
        unsafe { slice.get_unchecked_mut(index) }
    }};
}

#[macro_export]
macro_rules! unsafe_slice {
    ($s:expr,$r:expr) => {{
        let slice = &*$s;
        let index = $r;
        unsafe { slice.get_unchecked(index) }
    }};
}

pub mod buffer {
    use heapless::Vec;
    use zenoh_result::{bail, ZResult, ZE};

    use crate::cow::CowBytes;

    pub trait Buffer {
        /// Returns the number of bytes in the buffer.
        fn len(&self) -> usize;

        /// Returns `true` if the buffer has a length of 0.
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    /// A trait for buffers that can be composed of multiple non contiguous slices.
    pub trait SplitBuffer: Buffer {
        type Slices<'a>: Iterator<Item = &'a [u8]> + ExactSizeIterator
        where
            Self: 'a;

        /// Gets all the slices of this buffer.
        fn slices(&self) -> Self::Slices<'_>;

        /// Returns all the bytes of this buffer in a conitguous slice.
        /// This may require allocation and copy if the original buffer
        /// is not contiguous.
        fn contiguous<const N: usize>(&self) -> ZResult<CowBytes<'_, N>> {
            if self.len() > N {
                bail!(ZE::CapacityExceeded);
            }

            let mut slices = self.slices();
            match slices.len() {
                0 => Ok(CowBytes::Borrowed(b"")),
                1 => {
                    // SAFETY: unwrap here is safe because we have explicitly checked
                    //         the iterator has 1 element.
                    Ok(CowBytes::Borrowed(unsafe {
                        slices.next().unwrap_unchecked()
                    }))
                }
                _ => Ok(CowBytes::Owned(slices.fold(
                    Vec::<u8, N>::new(),
                    |mut acc, it| {
                        acc.extend_from_slice(it).expect("Capacity checked above");
                        acc
                    },
                ))),
            }
        }
    }
}

pub mod writer {
    use core::num::NonZeroUsize;

    use zenoh_result::ZResult;

    use crate::zslice::ZSlice;

    pub trait Writer {
        fn write(&mut self, bytes: &[u8]) -> ZResult<NonZeroUsize>;
        fn write_exact(&mut self, bytes: &[u8]) -> ZResult<()>;
        fn remaining(&self) -> usize;

        fn write_u8(&mut self, byte: u8) -> ZResult<()> {
            self.write_exact(core::slice::from_ref(&byte))
        }
        fn write_zslice(&mut self, slice: &ZSlice) -> ZResult<()> {
            self.write_exact(slice.as_slice())
        }
        fn can_write(&self) -> bool {
            self.remaining() != 0
        }
        /// Provides a buffer of exactly `len` uninitialized bytes to `write` to allow in-place writing.
        /// `write` must return the number of bytes it actually wrote.
        ///
        /// # Safety
        ///
        /// Caller must ensure that `write` return an integer lesser than or equal to the length of
        /// the slice passed in argument
        unsafe fn with_slot<F>(&mut self, len: usize, write: F) -> ZResult<NonZeroUsize>
        where
            F: FnOnce(&mut [u8]) -> usize;
    }

    impl<W: Writer + ?Sized> Writer for &mut W {
        fn write(&mut self, bytes: &[u8]) -> ZResult<NonZeroUsize> {
            (**self).write(bytes)
        }
        fn write_exact(&mut self, bytes: &[u8]) -> ZResult<()> {
            (**self).write_exact(bytes)
        }
        fn remaining(&self) -> usize {
            (**self).remaining()
        }
        fn write_u8(&mut self, byte: u8) -> ZResult<()> {
            (**self).write_u8(byte)
        }
        fn write_zslice(&mut self, slice: &ZSlice) -> ZResult<()> {
            (**self).write_zslice(slice)
        }
        fn can_write(&self) -> bool {
            (**self).can_write()
        }
        unsafe fn with_slot<F>(&mut self, len: usize, write: F) -> ZResult<NonZeroUsize>
        where
            F: FnOnce(&mut [u8]) -> usize,
        {
            unsafe { (**self).with_slot(len, write) }
        }
    }

    pub trait BacktrackableWriter: Writer {
        type Mark;

        fn mark(&mut self) -> Self::Mark;
        fn rewind(&mut self, mark: Self::Mark) -> bool;
    }

    impl<W: BacktrackableWriter + ?Sized> BacktrackableWriter for &mut W {
        type Mark = W::Mark;
        fn mark(&mut self) -> Self::Mark {
            (**self).mark()
        }
        fn rewind(&mut self, mark: Self::Mark) -> bool {
            (**self).rewind(mark)
        }
    }

    pub trait HasWriter {
        type Writer: Writer;

        /// Returns the most appropriate writer for `self`
        fn writer(self) -> Self::Writer;
    }
}

pub mod reader {
    use core::num::NonZeroUsize;

    use zenoh_result::{zerr, ZResult, ZE};

    use crate::zslice::ZSlice;

    pub trait Reader {
        fn read(&mut self, into: &mut [u8]) -> ZResult<NonZeroUsize>;
        fn read_exact(&mut self, into: &mut [u8]) -> ZResult<()>;
        fn remaining(&self) -> usize;

        /// Returns an iterator of `ZSlices` such that the sum of their length is _exactly_ `len`.
        fn read_zslices<F: FnMut(ZSlice), const N: usize>(
            &mut self,
            len: usize,
            for_each_slice: F,
        ) -> ZResult<()>;

        /// Reads exactly `len` bytes, returning them as a single `ZSlice`.
        fn read_zslice<const N: usize>(&mut self, len: usize) -> ZResult<ZSlice>;

        fn read_u8(&mut self) -> ZResult<u8> {
            let mut byte = 0;
            let read = self.read(core::slice::from_mut(&mut byte))?;
            if read.get() == 1 {
                Ok(byte)
            } else {
                Err(zerr!(ZE::DidntRead))
            }
        }

        fn can_read(&self) -> bool {
            self.remaining() != 0
        }
    }

    impl<R: Reader + ?Sized> Reader for &mut R {
        fn read(&mut self, into: &mut [u8]) -> ZResult<NonZeroUsize> {
            (**self).read(into)
        }
        fn read_exact(&mut self, into: &mut [u8]) -> ZResult<()> {
            (**self).read_exact(into)
        }
        fn remaining(&self) -> usize {
            (**self).remaining()
        }
        fn read_zslices<F: FnMut(ZSlice), const N: usize>(
            &mut self,
            len: usize,
            for_each_slice: F,
        ) -> ZResult<()> {
            (**self).read_zslices::<_, N>(len, for_each_slice)
        }
        fn read_zslice<const N: usize>(&mut self, len: usize) -> ZResult<ZSlice> {
            (**self).read_zslice::<N>(len)
        }
        fn read_u8(&mut self) -> ZResult<u8> {
            (**self).read_u8()
        }
        fn can_read(&self) -> bool {
            (**self).can_read()
        }
    }

    pub trait BacktrackableReader: Reader {
        type Mark;

        fn mark(&mut self) -> Self::Mark;
        fn rewind(&mut self, mark: Self::Mark) -> bool;
    }

    impl<R: BacktrackableReader + ?Sized> BacktrackableReader for &mut R {
        type Mark = R::Mark;
        fn mark(&mut self) -> Self::Mark {
            (**self).mark()
        }
        fn rewind(&mut self, mark: Self::Mark) -> bool {
            (**self).rewind(mark)
        }
    }

    pub trait AdvanceableReader: Reader {
        fn skip(&mut self, offset: usize) -> ZResult<()>;
        fn backtrack(&mut self, offset: usize) -> ZResult<()>;
        fn advance(&mut self, offset: isize) -> ZResult<()> {
            if offset > 0 {
                self.skip(offset as usize)
            } else {
                self.backtrack((-offset) as usize)
            }
        }
    }

    pub trait SiphonableReader: Reader {
        fn siphon<W>(&mut self, writer: &mut W) -> ZResult<NonZeroUsize>
        where
            W: crate::writer::Writer;
    }

    pub trait HasReader {
        type Reader: Reader;

        /// Returns the most appropriate reader for `self`
        fn reader(self) -> Self::Reader;
    }
}
