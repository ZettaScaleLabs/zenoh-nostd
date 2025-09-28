use core::{cmp, num::NonZeroUsize, ptr::NonNull};

use heapless::Vec;
use zenoh_result::{bail, zctx, zerr, WithContext, ZError, ZResult, ZE};

use crate::{
    buffer::{Buffer, SplitBuffer},
    reader::{AdvanceableReader, BacktrackableReader, HasReader, Reader, SiphonableReader},
    writer::{BacktrackableWriter, HasWriter, Writer},
    zslice::{ZSlice, ZSliceWriter},
};

#[derive(Debug, Clone, Default, Eq)]
pub struct ZBuf<const N: usize, const L: usize> {
    slices: Vec<ZSlice, N>,
    _slice_capacity: core::marker::PhantomData<[u8; L]>,
}

impl<const N: usize, const L: usize> ZBuf<N, L> {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            slices: Vec::new(),
            _slice_capacity: core::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.slices.clear();
    }

    pub fn zslices(&self) -> impl Iterator<Item = &ZSlice> + '_ {
        self.slices.iter()
    }

    pub fn zslices_mut(&mut self) -> impl Iterator<Item = &mut ZSlice> + '_ {
        self.slices.iter_mut()
    }

    pub fn into_zslices(self) -> impl Iterator<Item = ZSlice> {
        self.slices.into_iter()
    }

    pub fn push_zslice(&mut self, zslice: ZSlice) -> ZResult<()> {
        if !zslice.is_empty() {
            self.slices
                .push(zslice)
                .map_err(|_| zerr!(ZE::CapacityExceeded))
                .context(zctx!("Cannot push ZSlice into ZBuf"))?;
        }

        Ok(())
    }

    pub fn to_zslice<const M: usize>(&self) -> ZResult<ZSlice> {
        if M < L * N {
            bail!(ZE::InvalidArgument);
        }

        if self.len() > M {
            bail!(ZE::CapacityExceeded);
        }

        let mut slices = self.zslices();
        match self.slices.len() {
            0 => ZSlice::empty::<L>(),
            // SAFETY: it's safe to use unwrap_unchecked() because we are explicitly checking the length is 1.
            1 => unsafe { Ok(slices.next().unwrap_unchecked().clone()) },
            _ => slices
                .fold(Vec::<u8, M>::new(), |mut acc, it| {
                    acc.extend_from_slice(it.as_slice())
                        .expect("Capacity exceeded");
                    acc
                })
                .try_into(),
        }
    }

    #[inline]
    fn opt_zslice_writer(&mut self) -> Option<ZSliceWriter<'_, L>> {
        self.slices.last_mut().and_then(|s| s.writer())
    }
}

// Buffer
impl<const N: usize, const L: usize> Buffer for ZBuf<N, L> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.slices.iter().fold(0, |len, slice| len + slice.len())
    }
}

// SplitBuffer
impl<const N: usize, const L: usize> SplitBuffer for ZBuf<N, L> {
    type Slices<'a> = core::iter::Map<core::slice::Iter<'a, ZSlice>, fn(&'a ZSlice) -> &'a [u8]>;

    fn slices(&self) -> Self::Slices<'_> {
        self.slices.iter().map(ZSlice::as_slice)
    }
}

impl<const N: usize, const L: usize> PartialEq for ZBuf<N, L> {
    fn eq(&self, other: &Self) -> bool {
        let mut self_slices = self.slices();
        let mut other_slices = other.slices();
        let mut current_self = self_slices.next();
        let mut current_other = other_slices.next();
        loop {
            match (current_self, current_other) {
                (None, None) => return true,
                (None, _) | (_, None) => return false,
                (Some(l), Some(r)) => {
                    let cmp_len = l.len().min(r.len());
                    // SAFETY: cmp_len is the minimum length between l and r slices.
                    let lhs = crate::unsafe_slice!(l, ..cmp_len);
                    let rhs = crate::unsafe_slice!(r, ..cmp_len);
                    if lhs != rhs {
                        return false;
                    }
                    if cmp_len == l.len() {
                        current_self = self_slices.next();
                    } else {
                        // SAFETY: cmp_len is the minimum length between l and r slices.
                        let lhs = crate::unsafe_slice!(l, cmp_len..);
                        current_self = Some(lhs);
                    }
                    if cmp_len == r.len() {
                        current_other = other_slices.next();
                    } else {
                        // SAFETY: cmp_len is the minimum length between l and r slices.
                        let rhs = crate::unsafe_slice!(r, cmp_len..);
                        current_other = Some(rhs);
                    }
                }
            }
        }
    }
}

// From impls
impl<const N: usize, const L: usize> TryFrom<ZSlice> for ZBuf<N, L> {
    type Error = ZError;
    fn try_from(t: ZSlice) -> ZResult<Self> {
        let mut zbuf = ZBuf::empty();
        zbuf.push_zslice(t)
            .context(zctx!("Cannot convert ZSlice into ZBuf"))?;

        Ok(zbuf)
    }
}

impl<const N: usize, const L: usize> TryFrom<Vec<u8, L>> for ZBuf<N, L> {
    type Error = ZError;

    fn try_from(t: Vec<u8, L>) -> ZResult<Self> {
        let zslice: ZSlice = t
            .try_into()
            .context(zctx!("Cannot convert Vec into ZSlice"))?;

        Self::try_from(zslice)
    }
}

// Reader
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZBufPos {
    slice: usize,
    byte: usize,
}

#[derive(Debug, Clone)]
pub struct ZBufReader<'a, const N: usize, const L: usize> {
    inner: &'a ZBuf<N, L>,
    cursor: ZBufPos,
}

impl<'a, const N: usize, const L: usize> HasReader for &'a ZBuf<N, L> {
    type Reader = ZBufReader<'a, N, L>;

    fn reader(self) -> Self::Reader {
        ZBufReader {
            inner: self,
            cursor: ZBufPos { slice: 0, byte: 0 },
        }
    }
}

impl<const N: usize, const L: usize> Reader for ZBufReader<'_, N, L> {
    fn read(&mut self, mut into: &mut [u8]) -> ZResult<NonZeroUsize> {
        let mut read = 0;
        while let Some(slice) = self.inner.slices.get(self.cursor.slice) {
            // Subslice from the current read slice
            // SAFETY: validity of self.cursor.byte is ensured by the read logic.
            let from = crate::unsafe_slice!(slice.as_slice(), self.cursor.byte..);
            // Take the minimum length among read and write slices
            let len = from.len().min(into.len());
            // Copy the slice content
            // SAFETY: len is the minimum length between from and into slices.
            let lhs = crate::unsafe_slice_mut!(into, ..len);
            let rhs = crate::unsafe_slice!(from, ..len);
            lhs.copy_from_slice(rhs);
            // Advance the write slice
            // SAFETY: len is the minimum length between from and into slices.
            into = crate::unsafe_slice_mut!(into, len..);
            // Update the counter
            read += len;
            // Move the byte cursor
            self.cursor.byte += len;
            // We consumed all the current read slice, move to the next slice
            if self.cursor.byte == slice.len() {
                self.cursor.slice += 1;
                self.cursor.byte = 0;
            }
            // We have read everything we had to read
            if into.is_empty() {
                break;
            }
        }
        NonZeroUsize::new(read).ok_or(zerr!(ZE::DidntRead))
    }

    fn read_exact(&mut self, into: &mut [u8]) -> ZResult<()> {
        let len = Reader::read(self, into)?;
        if len.get() == into.len() {
            Ok(())
        } else {
            Err(zerr!(ZE::DidntRead))
        }
    }

    fn remaining(&self) -> usize {
        // SAFETY: self.cursor.slice validity is ensured by the reader
        let s = crate::unsafe_slice!(self.inner.slices, self.cursor.slice..);
        s.iter().fold(0, |acc, it| acc + it.len()) - self.cursor.byte
    }

    fn read_zslices<F: FnMut(ZSlice), const M: usize>(
        &mut self,
        len: usize,
        mut f: F,
    ) -> ZResult<()> {
        if len > L * N {
            bail!(ZE::CapacityExceeded);
        }
        if M != L {
            bail!(ZE::InvalidArgument);
        }

        if self.remaining() < len {
            return Err(zerr!(ZE::DidntRead));
        }

        let iter = ZBufSliceIterator {
            reader: self,
            remaining: len,
        };
        for slice in iter {
            f(slice);
        }

        Ok(())
    }

    fn read_zslice<const M: usize>(&mut self, len: usize) -> ZResult<ZSlice> {
        if len > L {
            bail!(ZE::CapacityExceeded);
        }

        if M != L {
            bail!(ZE::InvalidArgument);
        }

        let slice = self
            .inner
            .slices
            .get(self.cursor.slice)
            .ok_or(zerr!(ZE::DidntRead))?;

        match (slice.len() - self.cursor.byte).cmp(&len) {
            cmp::Ordering::Less => {
                let mut buffer = crate::vec::empty::<L>();
                buffer
                    .resize(len, 0)
                    .map_err(|_| zerr!(ZE::CapacityExceeded))
                    .context(zctx!("Cannot resize buffer"))?;

                Reader::read(self, &mut buffer)?;
                Ok(buffer.try_into()?)
            }
            cmp::Ordering::Equal => {
                let s = slice
                    .subslice(self.cursor.byte..)
                    .ok_or(zerr!(ZE::DidntRead))?;
                self.cursor.slice += 1;
                self.cursor.byte = 0;
                Ok(s)
            }
            cmp::Ordering::Greater => {
                let start = self.cursor.byte;
                self.cursor.byte += len;
                slice
                    .subslice(start..self.cursor.byte)
                    .ok_or(zerr!(ZE::DidntRead))
            }
        }
    }

    fn read_u8(&mut self) -> ZResult<u8> {
        let slice = self
            .inner
            .slices
            .get(self.cursor.slice)
            .ok_or(zerr!(ZE::DidntRead))?;

        let byte = *slice
            .get(self.cursor.byte)
            .ok_or(zerr!(ZE::DidntRead))
            .context(zctx!("Cannot read u8"))?;
        self.cursor.byte += 1;
        if self.cursor.byte == slice.len() {
            self.cursor.slice += 1;
            self.cursor.byte = 0;
        }
        Ok(byte)
    }

    fn can_read(&self) -> bool {
        self.inner.slices.get(self.cursor.slice).is_some()
    }
}

impl<const N: usize, const L: usize> BacktrackableReader for ZBufReader<'_, N, L> {
    type Mark = ZBufPos;

    fn mark(&mut self) -> Self::Mark {
        self.cursor
    }

    fn rewind(&mut self, mark: Self::Mark) -> bool {
        self.cursor = mark;
        true
    }
}

impl<const N: usize, const L: usize> SiphonableReader for ZBufReader<'_, N, L> {
    fn siphon<W>(&mut self, writer: &mut W) -> ZResult<NonZeroUsize>
    where
        W: Writer,
    {
        let mut read = 0;
        while let Some(slice) = self.inner.slices.get(self.cursor.slice) {
            // Subslice from the current read slice
            // SAFETY: self.cursor.byte is ensured by the reader.
            let from = crate::unsafe_slice!(slice.as_slice(), self.cursor.byte..);
            // Copy the slice content
            match writer.write(from) {
                Ok(len) => {
                    // Update the counter
                    read += len.get();
                    // Move the byte cursor
                    self.cursor.byte += len.get();
                    // We consumed all the current read slice, move to the next slice
                    if self.cursor.byte == slice.len() {
                        self.cursor.slice += 1;
                        self.cursor.byte = 0;
                    }
                }
                Err(_) => {
                    return NonZeroUsize::new(read).ok_or(
                        zerr!(ZE::DidntSiphon)
                            .context(zctx!("Cannot siphon from ZBufReader to Writer")),
                    );
                }
            }
        }
        NonZeroUsize::new(read)
            .ok_or(zerr!(ZE::DidntSiphon).context(zctx!("Cannot siphon from ZBufReader to Writer")))
    }
}

impl<const N: usize, const L: usize> AdvanceableReader for ZBufReader<'_, N, L> {
    fn skip(&mut self, offset: usize) -> ZResult<()> {
        let mut remaining_offset = offset;
        while remaining_offset > 0 {
            let s = self
                .inner
                .slices
                .get(self.cursor.slice)
                .ok_or(zerr!(ZE::DidntRead))?;
            let remains_in_current_slice = s.len() - self.cursor.byte;
            let advance = remaining_offset.min(remains_in_current_slice);
            remaining_offset -= advance;
            self.cursor.byte += advance;
            if self.cursor.byte == s.len() {
                self.cursor.slice += 1;
                self.cursor.byte = 0;
            }
        }
        Ok(())
    }

    fn backtrack(&mut self, offset: usize) -> ZResult<()> {
        let mut remaining_offset = offset;
        while remaining_offset > 0 {
            let backtrack = remaining_offset.min(self.cursor.byte);
            remaining_offset -= backtrack;
            self.cursor.byte -= backtrack;
            if self.cursor.byte == 0 {
                if self.cursor.slice == 0 {
                    break;
                }
                self.cursor.slice -= 1;
                self.cursor.byte = self
                    .inner
                    .slices
                    .get(self.cursor.slice)
                    .ok_or(zerr!(ZE::DidntRead))
                    .context(zctx!("Cannot backtrack ZBufReader"))?
                    .len();
            }
        }
        if remaining_offset == 0 {
            Ok(())
        } else {
            Err(zerr!(ZE::DidntRead)).context(zctx!("Cannot backtrack ZBufReader"))
        }
    }
}

// ZSlice iterator
pub struct ZBufSliceIterator<'a, 'b, const N: usize, const L: usize> {
    reader: &'a mut ZBufReader<'b, N, L>,
    remaining: usize,
}

impl<const N: usize, const L: usize> Iterator for ZBufSliceIterator<'_, '_, N, L> {
    type Item = ZSlice;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        // SAFETY: self.reader.cursor.slice is ensured by the reader.
        let slice = crate::unsafe_slice!(self.reader.inner.slices, self.reader.cursor.slice);
        let start = self.reader.cursor.byte;
        // SAFETY: self.reader.cursor.byte is ensured by the reader.
        let current = crate::unsafe_slice!(slice, start..);
        let len = current.len();
        match self.remaining.cmp(&len) {
            cmp::Ordering::Less => {
                let end = start + self.remaining;
                let slice = slice.subslice(start..end);
                self.reader.cursor.byte = end;
                self.remaining = 0;
                slice
            }
            cmp::Ordering::Equal => {
                let end = start + self.remaining;
                let slice = slice.subslice(start..end);
                self.reader.cursor.slice += 1;
                self.reader.cursor.byte = 0;
                self.remaining = 0;
                slice
            }
            cmp::Ordering::Greater => {
                let end = start + len;
                let slice = slice.subslice(start..end);
                self.reader.cursor.slice += 1;
                self.reader.cursor.byte = 0;
                self.remaining -= len;
                slice
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}

// Writer
#[derive(Debug)]
pub struct ZBufWriter<'a, const N: usize, const L: usize> {
    inner: NonNull<ZBuf<N, L>>,
    zslice_writer: Option<ZSliceWriter<'a, L>>,
}

impl<'a, const N: usize, const L: usize> ZBufWriter<'a, N, L> {
    #[inline]
    fn zslice_writer(&mut self) -> ZResult<&mut ZSliceWriter<'a, L>> {
        // Cannot use `if let` because of  https://github.com/rust-lang/rust/issues/54663
        if self.zslice_writer.is_some() {
            return Ok(self.zslice_writer.as_mut().unwrap());
        }
        // SAFETY: `self.inner` is valid as guaranteed by `self.writer` borrow
        let zbuf = unsafe { self.inner.as_mut() };
        zbuf.slices
            .push(ZSlice::empty::<L>()?)
            .map_err(|_| zerr!(ZE::CapacityExceeded))?;
        self.zslice_writer = zbuf.slices.last_mut().unwrap().writer();
        Ok(self.zslice_writer.as_mut().unwrap())
    }
}

impl<'a, const N: usize, const L: usize> HasWriter for &'a mut ZBuf<N, L> {
    type Writer = ZBufWriter<'a, N, L>;

    fn writer(self) -> Self::Writer {
        ZBufWriter {
            inner: NonNull::new(self).unwrap(),
            zslice_writer: self.opt_zslice_writer(),
        }
    }
}

impl<const N: usize, const L: usize> Writer for ZBufWriter<'_, N, L> {
    fn write(&mut self, bytes: &[u8]) -> ZResult<NonZeroUsize> {
        self.zslice_writer()?.write(bytes)
    }

    fn write_exact(&mut self, bytes: &[u8]) -> ZResult<()> {
        self.zslice_writer()?.write_exact(bytes)
    }

    fn remaining(&self) -> usize {
        usize::MAX
    }

    fn write_zslice(&mut self, slice: &ZSlice) -> ZResult<()> {
        self.zslice_writer = None;
        // SAFETY: `self.inner` is valid as guaranteed by `self.writer` borrow,
        // and `self.writer` has been overwritten
        unsafe { self.inner.as_mut() }.push_zslice(slice.clone())?;
        Ok(())
    }

    unsafe fn with_slot<F>(&mut self, len: usize, write: F) -> ZResult<NonZeroUsize>
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        // SAFETY: same precondition as the enclosing function
        self.zslice_writer()?.with_slot(len, write)
    }
}

impl<const N: usize, const L: usize> BacktrackableWriter for ZBufWriter<'_, N, L> {
    type Mark = ZBufPos;

    fn mark(&mut self) -> Self::Mark {
        let byte = self.zslice_writer.as_mut().map(|w| w.mark());
        // SAFETY: `self.inner` is valid as guaranteed by `self.writer` borrow
        let zbuf = unsafe { self.inner.as_mut() };
        ZBufPos {
            slice: zbuf.slices.len(),
            byte: byte
                .or_else(|| Some(zbuf.opt_zslice_writer()?.mark()))
                .unwrap_or(0),
        }
    }

    fn rewind(&mut self, mark: Self::Mark) -> bool {
        // SAFETY: `self.inner` is valid as guaranteed by `self.writer` borrow,
        // and `self.writer` is reassigned after modification
        let zbuf = unsafe { self.inner.as_mut() };
        zbuf.slices.truncate(mark.slice);
        self.zslice_writer = zbuf.opt_zslice_writer();
        if let Some(writer) = &mut self.zslice_writer {
            writer.rewind(mark.byte);
        }
        true
    }
}

mod tests {
    #[test]
    fn zbuf_eq() {
        use super::{ZBuf, ZSlice};
        use crate::zslice::ArcBytes16;
        use crate::{zslice::ArcBytes8, zunsafe_arc_pool_init};
        use heapless::Vec;

        zunsafe_arc_pool_init!(ArcBytes8: 2);
        zunsafe_arc_pool_init!(ArcBytes16: 1);

        let slice: ZSlice = Vec::<u8, 8>::from_array([0u8, 1, 2, 3, 4, 5, 6, 7])
            .try_into()
            .unwrap();

        let mut zbuf1 = ZBuf::<2, 4>::empty();
        zbuf1.push_zslice(slice.subslice(..4).unwrap()).unwrap();
        zbuf1.push_zslice(slice.subslice(4..8).unwrap()).unwrap();

        let mut zbuf2 = ZBuf::<3, 4>::empty();
        zbuf2.push_zslice(slice.subslice(..1).unwrap()).unwrap();
        zbuf2.push_zslice(slice.subslice(1..4).unwrap()).unwrap();
        zbuf2.push_zslice(slice.subslice(4..8).unwrap()).unwrap();

        assert_eq!(
            zbuf1.to_zslice::<8>().unwrap(),
            zbuf2.to_zslice::<16>().unwrap()
        );

        let mut zbuf1 = ZBuf::<2, 4>::empty();
        zbuf1.push_zslice(slice.subslice(2..4).unwrap()).unwrap();
        zbuf1.push_zslice(slice.subslice(4..8).unwrap()).unwrap();

        let mut zbuf2 = ZBuf::<3, 4>::empty();
        zbuf2.push_zslice(slice.subslice(2..3).unwrap()).unwrap();
        zbuf2.push_zslice(slice.subslice(3..6).unwrap()).unwrap();
        zbuf2.push_zslice(slice.subslice(6..8).unwrap()).unwrap();

        assert_eq!(
            zbuf1.to_zslice::<8>().unwrap(),
            zbuf2.to_zslice::<16>().unwrap()
        );
    }
}
