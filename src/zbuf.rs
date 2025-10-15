use crate::{result::ZResult, zbail};

crate::__internal_zerr! {
    /// An error that can occur during I/O operations.
    #[err = "io error"]
    enum ZIOError {
        DidNotRead,
        DidNotWrite,
        Invalid,
    }
}

pub type ZBuf<'a> = &'a [u8];
pub type ZBufMut<'a> = &'a mut [u8];
pub(crate) type ZBufReader<'a> = &'a [u8];
pub(crate) type ZBufWriter<'a> = &'a mut [u8];

pub(crate) trait ZBufExt<'a> {
    fn as_bytes(&self) -> &[u8];
    fn as_str(&self) -> ZResult<&'a str, ZIOError>;
    fn reader(&self) -> ZBufReader<'a>;
}

pub(crate) trait ZBufMutExt<'a> {
    fn as_bytes(&self) -> &[u8];
    fn as_str(&'a self) -> ZResult<&'a str, ZIOError>;
    fn reader(&'a self) -> ZBufReader<'a>;
    fn writer(&mut self) -> ZBufWriter<'_>;
    fn into_inner(self) -> ZBuf<'a>;
}

impl<'a> ZBufExt<'a> for ZBuf<'a> {
    fn as_bytes(&self) -> &[u8] {
        self
    }

    fn as_str(&self) -> ZResult<&'a str, ZIOError> {
        core::str::from_utf8(self).map_err(|_| ZIOError::Invalid)
    }

    fn reader(&self) -> ZBufReader<'a> {
        self
    }
}

impl<'a> ZBufMutExt<'a> for ZBufMut<'a> {
    fn as_bytes(&self) -> &[u8] {
        self
    }

    fn as_str(&'a self) -> ZResult<&'a str, ZIOError> {
        core::str::from_utf8(self).map_err(|_| ZIOError::Invalid)
    }

    fn writer(&mut self) -> ZBufWriter<'_> {
        self
    }

    fn reader(&'a self) -> ZBufReader<'a> {
        self
    }

    fn into_inner(self) -> ZBuf<'a> {
        self
    }
}

pub(crate) struct SliceMark<'s> {
    ptr: *const u8,
    len: usize,

    _phantom: core::marker::PhantomData<&'s [u8]>,
}

pub(crate) trait BufReaderExt<'a> {
    fn mark(&self) -> ZBuf<'a>;
    fn rewind(&mut self, mark: ZBuf<'a>);
    fn remaining(&self) -> usize;
    fn can_read(&self) -> bool;
    fn read_u8(&mut self) -> ZResult<u8, ZIOError>;
    fn read(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZIOError>;
    fn read_exact(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZIOError>;
    fn read_zbuf(&mut self, len: usize) -> ZResult<ZBuf<'a>, ZIOError>;
}

pub(crate) trait BufWriterExt<'a> {
    fn mark(&mut self) -> SliceMark<'a>;
    fn rewind(&mut self, mark: SliceMark<'a>);
    fn remaining(&self) -> usize;
    fn write_u8(&mut self, value: u8) -> ZResult<(), ZIOError>;
    fn write(&mut self, src: ZBuf<'_>) -> ZResult<usize, ZIOError>;
    fn write_exact(&mut self, src: ZBuf<'_>) -> ZResult<(), ZIOError>;
    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(ZBufMut<'_>) -> usize,
    ) -> ZResult<usize, ZIOError>;

    #[cfg(test)]
    fn write_slot_return(
        &mut self,
        len: usize,
        writer: impl FnOnce(ZBufMut<'_>) -> usize,
    ) -> ZResult<ZBuf<'a>, ZIOError>;

    #[cfg(test)]
    fn write_str_return(&mut self, str: &str) -> ZResult<&'a str, ZIOError>;
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

    fn read_u8(&mut self) -> ZResult<u8, ZIOError> {
        if !self.can_read() {
            zbail!(ZIOError::DidNotRead);
        }

        let value = self[0];
        *self = &self[1..];

        Ok(value)
    }

    fn read(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZIOError> {
        let len = self.remaining().min(dst.len());
        if len == 0 {
            zbail!(ZIOError::DidNotRead);
        }

        let (to_write, remain) = self.split_at(len);
        dst[..len].copy_from_slice(to_write);
        *self = remain;

        Ok(len)
    }

    fn read_exact(&mut self, dst: ZBufMut<'_>) -> ZResult<usize, ZIOError> {
        let len = dst.len();
        if self.len() < len {
            zbail!(ZIOError::DidNotRead);
        }

        let (to_write, remain) = self.split_at(len);
        dst.copy_from_slice(to_write);
        *self = remain;

        Ok(len)
    }

    fn read_zbuf(&mut self, len: usize) -> ZResult<ZBuf<'a>, ZIOError> {
        if self.len() < len {
            zbail!(ZIOError::DidNotRead);
        }

        let (zbuf, remain) = self.split_at(len);
        *self = remain;

        Ok(zbuf)
    }
}

impl<'a> BufWriterExt<'a> for ZBufWriter<'a> {
    fn mark(&mut self) -> SliceMark<'a> {
        SliceMark {
            ptr: self.as_ptr(),
            len: self.len(),
            _phantom: core::marker::PhantomData,
        }
    }

    fn rewind(&mut self, mark: SliceMark<'a>) {
        *self = unsafe { core::slice::from_raw_parts_mut(mark.ptr as *mut u8, mark.len) };
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn write_u8(&mut self, value: u8) -> ZResult<(), ZIOError> {
        self.write(&[value]).map(|_| ())
    }

    fn write(&mut self, src: &[u8]) -> ZResult<usize, ZIOError> {
        let len = self.len().min(src.len());
        if len == 0 {
            zbail!(ZIOError::DidNotWrite);
        }

        let (to_write, remain) = core::mem::take(self).split_at_mut(len);
        to_write.copy_from_slice(&src[..len]);
        *self = remain;

        Ok(len)
    }

    fn write_exact(&mut self, src: &[u8]) -> ZResult<(), ZIOError> {
        let len = src.len();
        if self.len() < len {
            zbail!(ZIOError::DidNotWrite);
        }

        self.write(src).map(|_| ())
    }

    fn write_slot(
        &mut self,
        len: usize,
        writer: impl FnOnce(&mut [u8]) -> usize,
    ) -> ZResult<usize, ZIOError> {
        if self.len() < len {
            zbail!(ZIOError::DidNotWrite);
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
    ) -> ZResult<ZBuf<'a>, ZIOError> {
        if self.len() < len {
            zbail!(ZIOError::DidNotWrite);
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
    fn write_str_return(&mut self, str: &str) -> ZResult<&'a str, ZIOError> {
        let bytes = str.as_bytes();

        let slot = self.write_slot_return(bytes.len(), |buf| {
            buf[..bytes.len()].copy_from_slice(bytes);
            bytes.len()
        })?;

        core::str::from_utf8(slot).map_err(|_| ZIOError::Invalid)
    }
}
