pub trait ZWrite {
    fn remaining(&self) -> usize;

    fn write(&mut self, src: &'_ [u8]) -> crate::ZResult<usize, crate::ZBytesError>;

    fn write_u8(&mut self, value: u8) -> crate::ZResult<(), crate::ZBytesError>;

    fn write_exact(&mut self, src: &'_ [u8]) -> crate::ZResult<(), crate::ZBytesError> {
        let len = src.len();
        let written = self.write(src)?;
        if written < len {
            return Err(crate::ZBytesError::DstIsTooSmall);
        }
        Ok(())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> crate::ZResult<usize, crate::ZBytesError>;
}

#[cfg(test)]
pub trait ZStore<'a>: ZWrite {
    type Mark;

    fn mark(&self) -> Self::Mark;
    unsafe fn slice(&self, mark: &Self::Mark) -> &'a [u8];

    unsafe fn store(
        &mut self,
        len: usize,
        data: impl FnOnce(&mut [u8]) -> usize,
    ) -> crate::ZResult<&'a [u8], crate::ZBytesError> {
        let mark = self.mark();
        let written = self.write_slot(len, data)?;
        Ok(unsafe { &self.slice(&mark)[..written] })
    }

    unsafe fn store_str(&mut self, s: &str) -> crate::ZResult<&'a str, crate::ZBytesError> {
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

pub trait ZRead<'a> {
    fn remaining(&self) -> usize;
    fn peek(&self) -> crate::ZResult<u8, crate::ZBytesError>;
    fn read_slice(&mut self, len: usize) -> crate::ZResult<&'a [u8], crate::ZBytesError>;

    fn can_read(&self) -> bool {
        self.remaining().gt(&0)
    }

    fn read_u8(&mut self) -> crate::ZResult<u8, crate::ZBytesError>;

    fn read(&mut self, dst: &'_ mut [u8]) -> crate::ZResult<usize, crate::ZBytesError> {
        let len = dst.len().min(self.remaining());
        let bytes = self.read_slice(len)?;

        unsafe {
            dst.get_unchecked_mut(..len).copy_from_slice(bytes);
        }

        Ok(len)
    }

    fn read_exact(&mut self, dst: &'_ mut [u8]) -> crate::ZResult<(), crate::ZBytesError> {
        let len = dst.len();
        if self.remaining() < len {
            crate::trace!(
                "read_exact ({}:{}:{}): src (len: {}) is too small to fill dst (len: {})",
                file!(),
                line!(),
                column!(),
                self.remaining(),
                len
            );
            return Err(crate::ZBytesError::SrcIsTooSmall);
        }
        let bytes = self.read_slice(len)?;
        dst.copy_from_slice(bytes);
        Ok(())
    }
}

impl ZWrite for &mut [u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> crate::ZResult<(), crate::ZBytesError> {
        if self.remaining() == 0 {
            crate::trace!(
                "write_u8 ({}:{}:{}): dst (len: 0) is full, cannot write value {}",
                file!(),
                line!(),
                column!(),
                value
            );
            return Err(crate::ZBytesError::DstIsFull);
        }

        unsafe {
            *self.get_unchecked_mut(0) = value;
            *self = ::core::mem::take(self).get_unchecked_mut(1..);
        }

        Ok(())
    }

    fn write(&mut self, src: &'_ [u8]) -> crate::ZResult<usize, crate::ZBytesError> {
        let len = src.len().min(self.len());
        let (head, tail) = unsafe { ::core::mem::take(self).split_at_mut_unchecked(len) };
        head.copy_from_slice(&src[..len]);
        *self = tail;
        Ok(len)
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> crate::ZResult<usize, crate::ZBytesError> {
        if self.len() < len {
            crate::trace!(
                "write_slot ({}:{}:{}): dst (len: {}) is too small to write slot of len {}",
                file!(),
                line!(),
                column!(),
                self.len(),
                len
            );
            return Err(crate::ZBytesError::DstIsTooSmall);
        }
        let written = writer(&mut self[..len]);
        if written > len {
            crate::trace!(
                "write_slot ({}:{}:{}): writer wrote {} bytes, which is more than the allocated slot size of {}",
                file!(),
                line!(),
                column!(),
                written,
                len
            );
            return Err(crate::ZBytesError::DstIsTooSmall);
        }
        let (_, tail) = unsafe { ::core::mem::take(self).split_at_mut_unchecked(written) };
        *self = tail;
        Ok(written)
    }
}

#[cfg(test)]
pub struct SliceMark<'a> {
    ptr: *const u8,
    len: usize,
    _marker: ::core::marker::PhantomData<&'a u8>,
}

#[cfg(test)]
impl<'a> ZStore<'a> for &'a mut [u8] {
    type Mark = SliceMark<'a>;
    fn mark(&self) -> Self::Mark {
        SliceMark {
            ptr: self.as_ptr(),
            len: self.len(),
            _marker: ::core::marker::PhantomData,
        }
    }

    unsafe fn slice(&self, mark: &Self::Mark) -> &'a [u8] {
        unsafe { ::core::slice::from_raw_parts_mut(mark.ptr as *mut u8, mark.len) }
    }
}

impl<'a> ZRead<'a> for &'a [u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn peek(&self) -> crate::ZResult<u8, crate::ZBytesError> {
        if self.can_read() {
            Ok(unsafe { *self.get_unchecked(0) })
        } else {
            crate::trace!(
                "peek ({}:{}:{}): src (len: 0) is empty, cannot peek",
                file!(),
                line!(),
                column!()
            );
            Err(crate::ZBytesError::SrcIsEmpty)
        }
    }

    fn read_u8(&mut self) -> crate::ZResult<u8, crate::ZBytesError> {
        if !self.can_read() {
            crate::trace!(
                "read_u8 ({}:{}:{}): src (len: 0) is empty, cannot read u8",
                file!(),
                line!(),
                column!()
            );
            return Err(crate::ZBytesError::SrcIsEmpty);
        }

        let value = unsafe { *self.get_unchecked(0) };
        *self = unsafe { self.get_unchecked(1..) };

        Ok(value)
    }

    fn read_slice(&mut self, len: usize) -> crate::ZResult<&'a [u8], crate::ZBytesError> {
        if self.remaining() < len {
            crate::trace!(
                "read_slice ({}:{}:{}): src (len: {}) is too small to read slice of len {}",
                file!(),
                line!(),
                column!(),
                self.remaining(),
                len
            );
            return Err(crate::ZBytesError::SrcIsTooSmall);
        }
        let (head, tail) = unsafe { self.split_at_unchecked(len) };
        *self = tail;
        Ok(head)
    }
}
