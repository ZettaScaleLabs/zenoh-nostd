pub trait ZWriteable {
    fn remaining(&self) -> usize;

    fn write(&mut self, src: &'_ [u8]) -> core::result::Result<usize, crate::BytesError>;

    fn write_u8(&mut self, value: u8) -> core::result::Result<(), crate::BytesError>;

    fn write_exact(&mut self, src: &'_ [u8]) -> core::result::Result<(), crate::BytesError> {
        let len = src.len();
        let written = self.write(src)?;
        if written < len {
            crate::error!(
                "dst (len: {}) is too small to write exact {} bytes - {}",
                self.remaining(),
                len,
                crate::zctx!()
            );

            crate::zbail!(crate::BytesError::DstIsTooSmall);
        }

        Ok(())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> core::result::Result<usize, crate::BytesError>;
}

#[cfg(test)]
pub trait ZStoreable<'a>: ZWriteable {
    type Mark;

    fn mark(&self) -> Self::Mark;
    unsafe fn slice(&self, mark: &Self::Mark) -> &'a [u8];

    unsafe fn store(
        &mut self,
        len: usize,
        data: impl FnOnce(&mut [u8]) -> usize,
    ) -> core::result::Result<&'a [u8], crate::BytesError> {
        let mark = self.mark();
        let written = self.write_slot(len, data)?;
        Ok(unsafe { &self.slice(&mark)[..written] })
    }

    unsafe fn store_str(&mut self, s: &str) -> core::result::Result<&'a str, crate::BytesError> {
        let bytes = s.as_bytes();
        let slot = unsafe {
            self.store(bytes.len(), |buf| {
                buf[..bytes.len()].copy_from_slice(bytes);
                bytes.len()
            })?
        };

        Ok(core::str::from_utf8(slot)
            .expect("Stored string is not valid UTF-8, this should never happen"))
    }
}

pub trait ZReadable<'a> {
    fn remaining(&self) -> usize;
    fn peek(&self) -> core::result::Result<u8, crate::BytesError>;
    fn read_slice(&mut self, len: usize) -> core::result::Result<&'a [u8], crate::BytesError>;

    fn can_read(&self) -> bool {
        self.remaining().gt(&0)
    }

    fn read_u8(&mut self) -> core::result::Result<u8, crate::BytesError>;

    fn read_exact(&mut self, dst: &'_ mut [u8]) -> core::result::Result<(), crate::BytesError> {
        let len = dst.len();
        if self.remaining() < len {
            crate::trace!(
                "src (len: {}) is too small to read exact {} bytes - {}",
                self.remaining(),
                len,
                crate::zctx!()
            );

            crate::zbail!(crate::BytesError::SrcIsTooSmall);
        }

        let bytes = self.read_slice(len)?;
        dst.copy_from_slice(bytes);

        Ok(())
    }
}

impl ZWriteable for &mut [u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> core::result::Result<(), crate::BytesError> {
        if self.remaining() == 0 {
            crate::trace!("dst (len: 0) is full, cannot write u8 - {}", crate::zctx!());
            crate::zbail!(crate::BytesError::DstIsFull);
        }

        unsafe {
            *self.get_unchecked_mut(0) = value;
            *self = core::mem::take(self).get_unchecked_mut(1..);
        }

        Ok(())
    }

    fn write(&mut self, src: &'_ [u8]) -> core::result::Result<usize, crate::BytesError> {
        let len = src.len().min(self.len());
        let (head, tail) = unsafe { core::mem::take(self).split_at_mut_unchecked(len) };

        head.copy_from_slice(&src[..len]);
        *self = tail;

        Ok(len)
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> core::result::Result<usize, crate::BytesError> {
        if self.len() < len {
            crate::trace!(
                "dst (len: {}) is too small to write slot of len {} - {}",
                self.len(),
                len,
                crate::zctx!()
            );
            crate::zbail!(crate::BytesError::DstIsTooSmall);
        }

        let written = writer(&mut self[..len]);

        if written > len {
            crate::trace!(
                "writer wrote {} bytes which exceeds the allocated slot size of {} - {}",
                written,
                len,
                crate::zctx!()
            );
            crate::zbail!(crate::BytesError::DstIsTooSmall);
        }

        let (_, tail) = unsafe { core::mem::take(self).split_at_mut_unchecked(written) };
        *self = tail;

        Ok(written)
    }
}

#[cfg(test)]
pub struct SliceMark<'a> {
    ptr: *const u8,
    len: usize,
    _marker: core::marker::PhantomData<&'a u8>,
}

#[cfg(test)]
impl<'a> ZStoreable<'a> for &'a mut [u8] {
    type Mark = SliceMark<'a>;
    fn mark(&self) -> Self::Mark {
        SliceMark {
            ptr: self.as_ptr(),
            len: self.len(),
            _marker: core::marker::PhantomData,
        }
    }

    unsafe fn slice(&self, mark: &Self::Mark) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(mark.ptr as *mut u8, mark.len) }
    }
}

impl<'a> ZReadable<'a> for &'a [u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn peek(&self) -> core::result::Result<u8, crate::BytesError> {
        if !self.can_read() {
            crate::trace!("src (len: 0) is empty, cannot peek u8 - {}", crate::zctx!());
            crate::zbail!(crate::BytesError::SrcIsEmpty)
        }

        Ok(unsafe { *self.get_unchecked(0) })
    }

    fn read_u8(&mut self) -> core::result::Result<u8, crate::BytesError> {
        if !self.can_read() {
            crate::trace!("src (len: 0) is empty, cannot read u8 - {}", crate::zctx!());
            crate::zbail!(crate::BytesError::SrcIsEmpty);
        }

        let value = unsafe { *self.get_unchecked(0) };
        *self = unsafe { self.get_unchecked(1..) };

        Ok(value)
    }

    fn read_slice(&mut self, len: usize) -> core::result::Result<&'a [u8], crate::BytesError> {
        if self.remaining() < len {
            crate::trace!(
                "src (len: {}) is too small to read {} bytes - {}",
                self.remaining(),
                len,
                crate::zctx!()
            );
            crate::zbail!(crate::BytesError::SrcIsTooSmall);
        }

        let (head, tail) = unsafe { self.split_at_unchecked(len) };
        *self = tail;

        Ok(head)
    }
}
