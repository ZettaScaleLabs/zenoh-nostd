#![no_std]

use zenoh_result::{zbail, zerr, ZResult, ZE};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZBuf<'a>(pub &'a [u8]);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZBufWithCapacity<'a, const N: usize> {
    pub buf: &'a [u8],
    _phantom: core::marker::PhantomData<[u8; N]>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ZBufMut<'a>(pub &'a mut [u8]);

impl<'a> ZBuf<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    pub fn as_str(&self) -> ZResult<&'a str> {
        core::str::from_utf8(self.0).map_err(|_| zerr!(ZE::FmtError))
    }

    pub fn split_at(&self, mid: usize) -> (ZBuf<'_>, ZBuf<'_>) {
        let (left, right) = self.0.split_at(mid);

        (ZBuf(left), ZBuf(right))
    }

    pub fn split_first(&self) -> Option<(&'_ u8, ZBuf<'_>)> {
        if self.0.is_empty() {
            None
        } else {
            Some((&self.0[0], ZBuf(&self.0[1..])))
        }
    }

    pub fn split_last(&self) -> Option<(ZBuf<'_>, &'_ u8)> {
        if self.0.is_empty() {
            None
        } else {
            Some((ZBuf(&self.0[..self.0.len() - 1]), &self.0[self.0.len() - 1]))
        }
    }

    pub fn subslice(&self, range: core::ops::Range<usize>) -> ZResult<ZBuf<'_>> {
        if range.start > range.end || range.end > self.0.len() {
            zbail!(ZE::CapacityExceeded);
        } else {
            Ok(ZBuf(&self.0[range]))
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        self.0.iter()
    }

    pub fn reader(&self) -> ZBufReader<'_> {
        ZBufReader {
            buf: ZBuf(self.0),
            pos: 0,
        }
    }
}

impl<'a> ZBufMut<'a> {
    pub fn from_bytes(bytes: &'a mut [u8]) -> Self {
        ZBufMut(bytes)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.0
    }

    pub fn split_at(&mut self, mid: usize) -> (ZBufMut<'_>, ZBufMut<'_>) {
        let (left, right) = self.0.split_at_mut(mid);

        (ZBufMut(left), ZBufMut(right))
    }

    pub fn split_first(&mut self) -> Option<(&'_ mut u8, ZBufMut<'_>)> {
        if self.0.is_empty() {
            None
        } else {
            let (first, rest) = self.0.split_first_mut().unwrap();
            Some((first, ZBufMut(rest)))
        }
    }

    pub fn split_last(&mut self) -> Option<(ZBufMut<'_>, &'_ mut u8)> {
        if self.0.is_empty() {
            None
        } else {
            let (last, rest) = self.0.split_last_mut().unwrap();
            Some((ZBufMut(rest), last))
        }
    }

    pub fn subslice(&mut self, range: core::ops::Range<usize>) -> ZResult<ZBufMut<'_>> {
        if range.start > range.end || range.end > self.0.len() {
            zbail!(ZE::CapacityExceeded);
        } else {
            Ok(ZBufMut(&mut self.0[range]))
        }
    }

    pub fn writer(&mut self) -> ZBufWriter<'_> {
        ZBufWriter {
            buf: ZBufMut(self.0),
            pos: 0,
        }
    }
}

pub struct ZBufWriter<'a> {
    buf: ZBufMut<'a>,
    pos: usize,
}

impl<'a> ZBufWriter<'a> {
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn write_u8(&mut self, value: u8) -> ZResult<usize> {
        if self.remaining() < 1 {
            zbail!(ZE::CapacityExceeded);
        }

        self.buf.0[self.pos] = value;
        self.pos += 1;

        Ok(1)
    }

    pub fn write(&mut self, src: &[u8], len: usize) -> ZResult<usize> {
        if self.remaining() < len || src.len() < len {
            zbail!(ZE::WriteFailure);
        }

        self.buf.0[self.pos..self.pos + len].copy_from_slice(&src[..len]);
        self.pos += len;

        Ok(len)
    }

    pub fn write_exact(&mut self, src: &[u8]) -> ZResult<usize> {
        let len = src.len();
        if self.remaining() < len {
            zbail!(ZE::WriteFailure);
        }

        self.buf.0[self.pos..self.pos + len].copy_from_slice(src);
        self.pos += len;

        Ok(len)
    }

    pub fn into_inner(self) -> ZBufMut<'a> {
        self.buf
    }
}

pub struct ZBufReader<'a> {
    buf: ZBuf<'a>,
    pos: usize,
}

impl<'a> ZBufReader<'a> {
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn can_read(&self) -> bool {
        self.remaining() > 0
    }

    pub fn read_u8(&mut self) -> ZResult<u8> {
        if self.remaining() < 1 {
            zbail!(ZE::CapacityExceeded);
        }

        let value = self.buf.0[self.pos];
        self.pos += 1;

        Ok(value)
    }

    pub fn read(&mut self, dst: &mut [u8], len: usize) -> ZResult<usize> {
        if self.remaining() < len || dst.len() < len {
            zbail!(ZE::ReadFailure);
        }

        dst[..len].copy_from_slice(&self.buf.0[self.pos..self.pos + len]);
        self.pos += len;

        Ok(len)
    }

    pub fn read_zbuf(&mut self, len: usize) -> ZResult<ZBuf<'a>> {
        if self.remaining() < len {
            zbail!(ZE::ReadFailure);
        }

        let zbuf = ZBuf(&self.buf.0[self.pos..self.pos + len]);
        self.pos += len;

        Ok(zbuf)
    }

    pub fn read_zbuf_with_capacity<const N: usize>(
        &mut self,
        len: usize,
    ) -> ZResult<ZBufWithCapacity<'_, N>> {
        if self.remaining() < len || len > N {
            zbail!(ZE::ReadFailure);
        }

        let zbuf = ZBufWithCapacity {
            buf: &self.buf.0[self.pos..self.pos + len],
            _phantom: core::marker::PhantomData,
        };

        self.pos += len;

        Ok(zbuf)
    }

    pub fn read_exact(&mut self, dst: &mut [u8]) -> ZResult<usize> {
        let len = dst.len();
        if self.remaining() < len {
            zbail!(ZE::ReadFailure);
        }

        dst.copy_from_slice(&self.buf.0[self.pos..self.pos + len]);
        self.pos += len;

        Ok(len)
    }

    pub fn into_inner(self) -> ZBuf<'a> {
        self.buf
    }
}
