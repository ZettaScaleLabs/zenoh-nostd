use crate::{result::ZResult, zbail};

crate::__internal_zerr! {
    /// An error that can occur during zbuf operations.
    #[err = "io buffer error"]
    enum ZBufError {
        CouldNotRead,
        CouldNotWrite,
        CouldNotParse,
    }
}

pub type ZBuf<'a> = &'a [u8];
pub type ZBufMut<'a> = &'a mut [u8];
pub(crate) type ZBufReader<'a> = &'a [u8];
pub(crate) type ZBufWriter<'a> = &'a mut [u8];

pub(crate) trait ZBufExt<'a> {
    fn as_str(&self) -> ZResult<&'a str, ZBufError>;
    fn reader(&self) -> ZBufReader<'a>;
}

pub(crate) trait ZBufMutExt<'a> {
    #[cfg(test)]
    fn reader(&'a self) -> ZBufReader<'a>;
    fn writer(&mut self) -> ZBufWriter<'_>;
}

impl<'a> ZBufExt<'a> for ZBuf<'a> {
    fn as_str(&self) -> ZResult<&'a str, ZBufError> {
        core::str::from_utf8(self).map_err(|_| ZBufError::CouldNotParse)
    }

    fn reader(&self) -> ZBufReader<'a> {
        self
    }
}

impl<'a> ZBufMutExt<'a> for ZBufMut<'a> {
    fn writer(&mut self) -> ZBufWriter<'_> {
        self
    }

    #[cfg(test)]
    fn reader(&'a self) -> ZBufReader<'a> {
        self
    }
}

pub(crate) trait BufReaderExt<'a> {
    fn mark(&self) -> ZBuf<'a>;
    fn rewind(&mut self, mark: ZBuf<'a>);
    fn remaining(&self) -> usize;
    fn can_read(&self) -> bool;
    fn read_u8(&mut self) -> ZResult<u8, ZBufError>;
    fn read(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZBufError>;
    fn read_zbuf(&mut self, len: usize) -> ZResult<ZBuf<'a>, ZBufError>;
}

pub(crate) trait BufWriterExt<'a> {
    fn remaining(&self) -> usize;
    fn write_u8(&mut self, value: u8) -> ZResult<(), ZBufError>;
    fn write(&mut self, src: ZBuf<'_>) -> ZResult<usize, ZBufError>;
    fn write_exact(&mut self, src: ZBuf<'_>) -> ZResult<(), ZBufError>;
    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(ZBufMut<'_>) -> usize,
    ) -> ZResult<usize, ZBufError>;

    #[cfg(test)]
    fn write_slot_return(
        &mut self,
        len: usize,
        writer: impl FnOnce(ZBufMut<'_>) -> usize,
    ) -> ZResult<ZBuf<'a>, ZBufError>;

    #[cfg(test)]
    fn write_str_return(&mut self, str: &str) -> ZResult<&'a str, ZBufError>;
}

impl<'a> BufReaderExt<'a> for ZBufReader<'a> {
    fn mark(&self) -> ZBuf<'a> {
        self
    }

    fn rewind(&mut self, mark: ZBuf<'a>) {
        *self = mark;
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn can_read(&self) -> bool {
        !self.is_empty()
    }

    fn read_u8(&mut self) -> ZResult<u8, ZBufError> {
        if !self.can_read() {
            zbail!(ZBufError::CouldNotRead);
        }

        let value = self[0];
        *self = &self[1..];

        Ok(value)
    }

    fn read(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZBufError> {
        let len = self.remaining().min(dst.len());
        if len == 0 {
            zbail!(ZBufError::CouldNotRead);
        }

        let (to_write, remain) = self.split_at(len);
        dst[..len].copy_from_slice(to_write);
        *self = remain;

        Ok(len)
    }

    fn read_zbuf(&mut self, len: usize) -> ZResult<ZBuf<'a>, ZBufError> {
        if self.len() < len {
            zbail!(ZBufError::CouldNotRead);
        }

        let (zbuf, remain) = self.split_at(len);
        *self = remain;

        Ok(zbuf)
    }
}

impl<'a> BufWriterExt<'a> for ZBufWriter<'a> {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> ZResult<(), ZBufError> {
        self.write(&[value]).map(|_| ())
    }

    fn write(&mut self, src: &[u8]) -> ZResult<usize, ZBufError> {
        let len = self.len().min(src.len());
        if len == 0 {
            zbail!(ZBufError::CouldNotWrite);
        }

        let (to_write, remain) = core::mem::take(self).split_at_mut(len);
        to_write.copy_from_slice(&src[..len]);
        *self = remain;

        Ok(len)
    }

    fn write_exact(&mut self, src: &[u8]) -> ZResult<(), ZBufError> {
        let len = src.len();
        if self.len() < len {
            zbail!(ZBufError::CouldNotWrite);
        }

        self.write(src).map(|_| ())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> ZResult<usize, ZBufError> {
        if self.len() < len {
            zbail!(ZBufError::CouldNotWrite);
        }

        let written = writer(&mut self[..len]);

        *self = unsafe { core::mem::take(self).get_unchecked_mut(written..) };

        Ok(written)
    }

    #[cfg(test)]
    fn write_slot_return(
        &mut self,
        len: usize,
        writer: impl FnOnce(ZBufMut<'_>) -> usize,
    ) -> ZResult<ZBuf<'a>, ZBufError> {
        if self.len() < len {
            zbail!(ZBufError::CouldNotWrite);
        }

        let written = writer(&mut self[..len]);

        let ret = unsafe {
            let (ret, remain) = core::mem::take(self).split_at_mut_unchecked(written);
            *self = remain;
            ret
        };

        Ok(ret)
    }

    #[cfg(test)]
    fn write_str_return(&mut self, str: &str) -> ZResult<&'a str, ZBufError> {
        let bytes = str.as_bytes();

        let slot = self.write_slot_return(bytes.len(), |buf| {
            buf[..bytes.len()].copy_from_slice(bytes);
            bytes.len()
        })?;

        slot.as_str()
    }
}
