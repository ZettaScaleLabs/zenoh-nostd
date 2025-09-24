use core::{mem, num::NonZeroUsize, option};

use heapless::Vec;
use zenoh_result::{bail, zerr, ZResult, ZE};

use crate::{
    buffer::{Buffer, SplitBuffer},
    reader::HasReader,
    writer::{BacktrackableWriter, HasWriter, Writer},
};

/// Allocate a vector with a given capacity and sets the length to that capacity.
#[must_use]
pub fn uninit<const N: usize>() -> Vec<u8, N> {
    let mut vbuf = Vec::new();
    // SAFETY: this operation is safe since we are setting the length equal to the allocated capacity.
    #[allow(clippy::uninit_vec)]
    unsafe {
        vbuf.set_len(N);
    }
    vbuf
}

/// Allocate a vector with a given capacity.
#[must_use]
pub fn empty<const N: usize>() -> Vec<u8, N> {
    Vec::new()
}

// Buffer
impl<const N: usize> Buffer for Vec<u8, N> {
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl<const N: usize> Buffer for &Vec<u8, N> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<const N: usize> Buffer for &mut Vec<u8, N> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

// SplitBuffer
impl<const N: usize> SplitBuffer for Vec<u8, N> {
    type Slices<'a> = option::IntoIter<&'a [u8]>;

    fn slices(&self) -> Self::Slices<'_> {
        Some(self.as_slice()).into_iter()
    }
}

// Writer
impl<const N: usize> HasWriter for &mut Vec<u8, N> {
    type Writer = Self;

    fn writer(self) -> Self::Writer {
        self
    }
}

impl<const N: usize> Writer for Vec<u8, N> {
    fn write(&mut self, bytes: &[u8]) -> ZResult<NonZeroUsize> {
        if bytes.is_empty() {
            bail!(ZE::DidntWrite);
        }

        self.extend_from_slice(bytes)
            .map_err(|_| zerr!(ZE::DidntWrite))?;
        // SAFETY: this operation is safe since we early return in case bytes is empty
        Ok(unsafe { NonZeroUsize::new_unchecked(bytes.len()) })
    }

    fn write_exact(&mut self, bytes: &[u8]) -> ZResult<()> {
        self.write(bytes).map(|_| ())
    }

    fn remaining(&self) -> usize {
        usize::MAX
    }

    fn write_u8(&mut self, byte: u8) -> ZResult<()> {
        self.push(byte).map_err(|_| zerr!(ZE::DidntWrite))?;
        Ok(())
    }

    unsafe fn with_slot<F>(&mut self, mut len: usize, write: F) -> ZResult<NonZeroUsize>
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        if N < self.len() + len {
            bail!(ZE::DidntWrite);
        }

        // SAFETY: we already reserved len elements on the vector.
        let s = crate::unsafe_slice_mut!(self.spare_capacity_mut(), ..len);
        // SAFETY: converting MaybeUninit<u8> into [u8] is safe because we are going to write on it.
        //         The returned len tells us how many bytes have been written so as to update the len accordingly.
        len = unsafe { write(&mut *(s as *mut [mem::MaybeUninit<u8>] as *mut [u8])) };
        // SAFETY: we already reserved len elements on the vector.
        unsafe { self.set_len(self.len() + len) };

        NonZeroUsize::new(len).ok_or(zerr!(ZE::DidntWrite))
    }
}

impl<const N: usize> BacktrackableWriter for Vec<u8, N> {
    type Mark = usize;

    fn mark(&mut self) -> Self::Mark {
        self.len()
    }

    fn rewind(&mut self, mark: Self::Mark) -> bool {
        self.truncate(mark);
        true
    }
}

// Reader
impl<'a, const N: usize> HasReader for &'a Vec<u8, N> {
    type Reader = &'a [u8];

    fn reader(self) -> Self::Reader {
        self
    }
}
