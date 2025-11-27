pub mod r#struct;
pub use r#struct::*;

pub mod ext;
pub use ext::*;

pub type ZReader<'a> = &'a [u8];
pub type ZWriter<'a> = &'a mut [u8];

pub trait ZReaderExt<'a> {
    fn mark(&self) -> &'a [u8];
    fn rewind(&mut self, mark: &'a [u8]);

    fn remaining(&self) -> usize;
    fn can_read(&self) -> bool {
        self.remaining().gt(&0)
    }

    fn peek_u8(&self) -> crate::ZResult<u8, crate::ZCodecError>;

    fn read(&mut self, len: usize) -> crate::ZResult<&'a [u8], crate::ZCodecError>;
    fn read_u8(&mut self) -> crate::ZResult<u8, crate::ZCodecError>;

    fn read_into(&mut self, dst: &'_ mut [u8]) -> crate::ZResult<usize, crate::ZCodecError>;

    fn sub(&mut self, len: usize) -> crate::ZResult<ZReader<'a>, crate::ZCodecError> {
        let sub = self.read(len)?;
        Ok(sub)
    }
}

pub trait ZWriterExt<'a> {
    fn write(&mut self, src: &'_ [u8]) -> crate::ZResult<usize, crate::ZCodecError>;
    fn write_u8(&mut self, value: u8) -> crate::ZResult<(), crate::ZCodecError>;

    fn write_exact(&mut self, src: &'_ [u8]) -> crate::ZResult<(), crate::ZCodecError>;
    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&'_ mut [u8]) -> usize,
    ) -> crate::ZResult<&'a [u8], crate::ZCodecError>;

    #[cfg(test)]
    fn write_str(&mut self, s: &str) -> crate::ZResult<&'a str, crate::ZCodecError> {
        let bytes = s.as_bytes();
        let slot = self.write_slot(bytes.len(), |buf| {
            buf[..bytes.len()].copy_from_slice(bytes);
            bytes.len()
        })?;

        core::str::from_utf8(slot).map_err(|_| crate::ZCodecError::CouldNotWrite)
    }
}

impl<'a> ZReaderExt<'a> for ZReader<'a> {
    fn mark(&self) -> &'a [u8] {
        self
    }

    fn rewind(&mut self, mark: &'a [u8]) {
        *self = mark;
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn peek_u8(&self) -> crate::ZResult<u8, crate::ZCodecError> {
        if !self.can_read() {
            return Err(crate::ZCodecError::CouldNotRead);
        }

        Ok(unsafe { *self.get_unchecked(0) })
    }

    fn read_u8(&mut self) -> crate::ZResult<u8, crate::ZCodecError> {
        if !self.can_read() {
            return Err(crate::ZCodecError::CouldNotRead);
        }

        let value = unsafe { *self.get_unchecked(0) };
        *self = unsafe { self.get_unchecked(1..) };

        Ok(value)
    }

    fn read_into(&mut self, dst: &'_ mut [u8]) -> crate::ZResult<usize, crate::ZCodecError> {
        let len = self.remaining().min(dst.len());
        if len == 0 {
            return Err(crate::ZCodecError::CouldNotRead);
        }

        let (to_write, remain) = unsafe { self.split_at_unchecked(len) };
        unsafe {
            dst.get_unchecked_mut(..len).copy_from_slice(to_write);
        }

        *self = remain;

        Ok(len)
    }

    fn read(&mut self, len: usize) -> crate::ZResult<&'a [u8], crate::ZCodecError> {
        if self.len() < len {
            return Err(crate::ZCodecError::CouldNotRead);
        }

        let (zbuf, remain) = unsafe { self.split_at_unchecked(len) };
        *self = remain;

        Ok(zbuf)
    }
}

impl<'a> ZWriterExt<'a> for ZWriter<'a> {
    fn write_u8(&mut self, value: u8) -> crate::ZResult<(), crate::ZCodecError> {
        if self.is_empty() {
            return Err(crate::ZCodecError::CouldNotWrite);
        }

        unsafe {
            *self.get_unchecked_mut(0) = value;
            *self = core::mem::take(self).get_unchecked_mut(1..);
        }

        Ok(())
    }

    fn write(&mut self, src: &[u8]) -> crate::ZResult<usize, crate::ZCodecError> {
        if src.is_empty() {
            return Ok(0);
        }
        let len = self.len().min(src.len());
        if len == 0 {
            return Err(crate::ZCodecError::CouldNotWrite);
        }

        let (to_write, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(len) };
        to_write.copy_from_slice(unsafe { src.get_unchecked(..len) });
        *self = remain;

        Ok(len)
    }

    fn write_exact(&mut self, src: &[u8]) -> crate::ZResult<(), crate::ZCodecError> {
        let len = self.write(src)?;

        if len < src.len() {
            return Err(crate::ZCodecError::CouldNotWrite);
        }

        Ok(())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> crate::ZResult<&'a [u8], crate::ZCodecError> {
        if self.len() < len {
            return Err(crate::ZCodecError::CouldNotWrite);
        }

        let written = writer(unsafe { self.get_unchecked_mut(..len) });

        if written > len {
            return Err(crate::ZCodecError::CouldNotWrite);
        }

        let (slot, remain) = unsafe { core::mem::take(self).split_at_mut_unchecked(written) };
        *self = remain;

        Ok(slot)
    }
}
