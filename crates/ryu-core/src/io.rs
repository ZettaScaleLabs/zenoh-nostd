pub type ByteReader<'a> = &'a [u8];
pub type ByteWriter<'a> = &'a mut [u8];

crate::__internal_err! {
    /// Errors related to IO operations on byte buffers
    #[err = "byte IO error"]
    enum ByteIOError {
        CouldNotRead,
        CouldNotWrite,
        CouldNotParse
    }
}

pub type ByteIOResult<T> = crate::result::Result<T, ByteIOError>;

pub trait ByteReaderExt<'a> {
    fn mark(&self) -> &'a [u8];
    fn rewind(&mut self, mark: &'a [u8]);

    fn remaining(&self) -> usize;
    fn can_read(&self) -> bool {
        self.remaining().gt(&0)
    }

    fn peek_u8(&self) -> ByteIOResult<u8>;

    fn read(&mut self, len: usize) -> ByteIOResult<&'a [u8]>;
    fn read_u8(&mut self) -> ByteIOResult<u8>;

    fn read_into(&mut self, dst: &'_ mut [u8]) -> ByteIOResult<usize>;

    fn sub(&mut self, len: usize) -> ByteIOResult<ByteReader<'a>> {
        let sub = self.read(len)?;
        Ok(sub)
    }
}

pub trait ByteWriterExt<'a> {
    fn remaining(&self) -> usize;

    fn write(&mut self, src: &'_ [u8]) -> ByteIOResult<usize>;
    fn write_u8(&mut self, value: u8) -> ByteIOResult<()>;

    fn write_exact(&mut self, src: &'_ [u8]) -> ByteIOResult<()>;
    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&'_ mut [u8]) -> usize,
    ) -> ByteIOResult<&'a [u8]>;
}

impl<'a> ByteReaderExt<'a> for ByteReader<'a> {
    fn mark(&self) -> &'a [u8] {
        self
    }

    fn rewind(&mut self, mark: &'a [u8]) {
        *self = mark;
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn peek_u8(&self) -> ByteIOResult<u8> {
        if !self.can_read() {
            return Err(ByteIOError::CouldNotRead);
        }

        Ok(unsafe { *self.get_unchecked(0) })
    }

    fn read_u8(&mut self) -> ByteIOResult<u8> {
        if !self.can_read() {
            return Err(ByteIOError::CouldNotRead);
        }

        let value = unsafe { *self.get_unchecked(0) };
        *self = unsafe { self.get_unchecked(1..) };

        Ok(value)
    }

    fn read_into(&mut self, dst: &'_ mut [u8]) -> ByteIOResult<usize> {
        if dst.is_empty() {
            return Ok(0);
        }

        let len = self.remaining().min(dst.len());
        if len == 0 {
            return Err(ByteIOError::CouldNotRead);
        }

        let (to_write, remain) = unsafe { self.split_at_unchecked(len) };
        unsafe {
            dst.get_unchecked_mut(..len).copy_from_slice(to_write);
        }

        *self = remain;

        Ok(len)
    }

    fn read(&mut self, len: usize) -> ByteIOResult<&'a [u8]> {
        if self.len() < len {
            return Err(ByteIOError::CouldNotRead);
        }

        let (zbuf, remain) = unsafe { self.split_at_unchecked(len) };
        *self = remain;

        Ok(zbuf)
    }
}

impl<'a> ByteWriterExt<'a> for ByteWriter<'a> {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> ByteIOResult<()> {
        if self.is_empty() {
            return Err(ByteIOError::CouldNotWrite);
        }

        unsafe {
            *self.get_unchecked_mut(0) = value;
            *self = core::mem::take(self).get_unchecked_mut(1..);
        }

        Ok(())
    }

    fn write(&mut self, src: &[u8]) -> ByteIOResult<usize> {
        if src.is_empty() {
            return Ok(0);
        }

        let len = self.len().min(src.len());
        if len == 0 {
            return Err(ByteIOError::CouldNotWrite);
        }

        let (to_write, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(len) };
        to_write.copy_from_slice(unsafe { src.get_unchecked(..len) });
        *self = remain;

        Ok(len)
    }

    fn write_exact(&mut self, src: &[u8]) -> ByteIOResult<()> {
        let len = self.write(src)?;

        if len < src.len() {
            return Err(ByteIOError::CouldNotWrite);
        }

        Ok(())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> ByteIOResult<&'a [u8]> {
        if self.len() < len {
            return Err(ByteIOError::CouldNotWrite);
        }

        let written = writer(unsafe { self.get_unchecked_mut(..len) });

        if written > len {
            return Err(ByteIOError::CouldNotWrite);
        }

        let (slot, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(written) };
        *self = remain;

        Ok(slot)
    }
}
