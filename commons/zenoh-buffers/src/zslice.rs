use core::{
    fmt, hash,
    num::NonZeroUsize,
    ops::{Bound, Deref, RangeBounds},
};

use heapless::{arc_pool, pool::arc::Arc, Vec};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::{
    buffer::{Buffer, SplitBuffer},
    reader::{BacktrackableReader, HasReader, Reader},
    writer::{BacktrackableWriter, Writer},
};

/// Macro pour déclarer les arc_pools
macro_rules! declare_arc_pools {
    ($($arc_ty:ident: $size:expr),* $(,)?) => {
        $(
            arc_pool!($arc_ty: Vec<u8, $size>);
        )*
    };
}

/// Macro pour l'enum ZSlice et TryFrom
macro_rules! declare_abuf_variants {
    ($($size:expr => $variant:ident => $arc_ty:ident),* $(,)?) => {
        #[derive(Clone)]
        pub enum ABuf {
            $(
                $variant(Arc<$arc_ty>),
            )*
        }

        impl<const N: usize> TryFrom<Vec<u8, N>> for ABuf {
            type Error = ZError;

            fn try_from(value: Vec<u8, N>) -> ZResult<Self> {
                match N {
                    $(
                        $size => Ok(ABuf::$variant(
                            $arc_ty
                                .alloc(Self::transmut(value)?)
                                .map_err(|_| zerr!(ZE::CapacityExceeded))?,
                        )),
                    )*
                    _ => bail!(ZE::InvalidArgument),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! zunsafe_arc_pool_init {
    ($pool:ident : $capacity:expr) => {{
        use core::ptr::addr_of_mut;
        use heapless::pool::arc::ArcBlock;

        const _POOL_CAPACITY: usize = $capacity;
        type BlockType = <$pool as heapless::pool::arc::ArcPool>::Data;
        const _INIT_BLOCK: ArcBlock<BlockType> = ArcBlock::new();

        static mut _BLOCKS: [ArcBlock<BlockType>; _POOL_CAPACITY] = [_INIT_BLOCK; _POOL_CAPACITY];

        let blocks: &'static mut [ArcBlock<BlockType>] =
            unsafe { addr_of_mut!(_BLOCKS).as_mut().unwrap() };

        for block in blocks {
            $pool.manage(block);
        }
    }};
}

// Déclaration des pools
declare_arc_pools! {
    ArcBytes2: 2,
    ArcBytes4: 4,
    ArcBytes8: 8,
    ArcBytes16: 16,
    ArcBytes32: 32,
    ArcBytes64: 64,
    ArcBytes128: 128,
    ArcBytes256: 256,
    ArcBytes512: 512,
    ArcBytes1024: 1024,
    ArcBytes2048: 2048,
    ArcBytes4096: 4096,
    ArcBytes8192: 8192,
    ArcBytes16384: 16384,
    ArcBytes32768: 32768,
    ArcBytes65536: 65536,
}

// Déclaration de ZSlice avec mapping enum <-> pool
declare_abuf_variants! {
    2 => Bytes2 => ArcBytes2,
    4 => Bytes4 => ArcBytes4,
    8 => Bytes8 => ArcBytes8,
    16 => Bytes16 => ArcBytes16,
    32 => Bytes32 => ArcBytes32,
    64 => Bytes64 => ArcBytes64,
    128 => Bytes128 => ArcBytes128,
    256 => Bytes256 => ArcBytes256,
    512 => Bytes512 => ArcBytes512,
    1024 => Bytes1024 => ArcBytes1024,
    2048 => Bytes2048 => ArcBytes2048,
    4096 => Bytes4096 => ArcBytes4096,
    8192 => Bytes8192 => ArcBytes8192,
    16384 => Bytes16384 => ArcBytes16384,
    32768 => Bytes32768 => ArcBytes32768,
    65536 => Bytes65536 => ArcBytes65536,
}

impl ABuf {
    fn transmut<const N: usize, const M: usize>(vec: Vec<u8, N>) -> ZResult<Vec<u8, M>> {
        if N != M {
            bail!(ZE::InvalidArgument);
        }

        let ptr = &vec as *const Vec<u8, N> as *const Vec<u8, M>;
        let res = unsafe { ptr.read() };
        core::mem::forget(vec);

        Ok(res)
    }

    fn transmut_mut<const N: usize, const M: usize>(
        vec: &mut Vec<u8, N>,
    ) -> ZResult<&mut Vec<u8, M>> {
        if N != M {
            bail!(ZE::InvalidArgument);
        }

        let ptr = vec as *mut Vec<u8, N> as *mut Vec<u8, M>;
        let res = unsafe { &mut *ptr };

        Ok(res)
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Bytes2(v) => v.as_slice(),
            Self::Bytes4(v) => v.as_slice(),
            Self::Bytes8(v) => v.as_slice(),
            Self::Bytes16(v) => v.as_slice(),
            Self::Bytes32(v) => v.as_slice(),
            Self::Bytes64(v) => v.as_slice(),
            Self::Bytes128(v) => v.as_slice(),
            Self::Bytes256(v) => v.as_slice(),
            Self::Bytes512(v) => v.as_slice(),
            Self::Bytes1024(v) => v.as_slice(),
            Self::Bytes2048(v) => v.as_slice(),
            Self::Bytes4096(v) => v.as_slice(),
            Self::Bytes8192(v) => v.as_slice(),
            Self::Bytes16384(v) => v.as_slice(),
            Self::Bytes32768(v) => v.as_slice(),
            Self::Bytes65536(v) => v.as_slice(),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Bytes2(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes4(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes8(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes16(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes32(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes64(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes128(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes256(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes512(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes1024(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes2048(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes4096(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes8192(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes16384(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes32768(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
            Self::Bytes65536(v) => Arc::get_mut(v).map(|v| v.as_mut_slice()),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_mut_vec<const N: usize>(&mut self) -> Option<&mut Vec<u8, N>> {
        match self {
            Self::Bytes2(v) if N == 2 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<2, N>(v).unwrap())
            }
            Self::Bytes4(v) if N == 4 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<4, N>(v).unwrap())
            }
            Self::Bytes8(v) if N == 8 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<8, N>(v).unwrap())
            }
            Self::Bytes16(v) if N == 16 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<16, N>(v).unwrap())
            }
            Self::Bytes32(v) if N == 32 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<32, N>(v).unwrap())
            }
            Self::Bytes64(v) if N == 64 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<64, N>(v).unwrap())
            }
            Self::Bytes128(v) if N == 128 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<128, N>(v).unwrap())
            }
            Self::Bytes256(v) if N == 256 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<256, N>(v).unwrap())
            }
            Self::Bytes512(v) if N == 512 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<512, N>(v).unwrap())
            }
            Self::Bytes1024(v) if N == 1024 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<1024, N>(v).unwrap())
            }
            Self::Bytes2048(v) if N == 2048 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<2048, N>(v).unwrap())
            }
            Self::Bytes4096(v) if N == 4096 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<4096, N>(v).unwrap())
            }
            Self::Bytes8192(v) if N == 8192 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<8192, N>(v).unwrap())
            }
            Self::Bytes16384(v) if N == 16384 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<16384, N>(v).unwrap())
            }
            Self::Bytes32768(v) if N == 32768 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<32768, N>(v).unwrap())
            }
            Self::Bytes65536(v) if N == 65536 => {
                Arc::get_mut(v).map(|v| Self::transmut_mut::<65536, N>(v).unwrap())
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ZSlice {
    buf: ABuf,
    start: usize,
    end: usize,
}

impl ZSlice {
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: bounds checks are performed at `ZSlice` construction.
        unsafe { self.buf.as_slice().get_unchecked(self.start..self.end) }
    }

    #[inline]
    pub fn empty<const N: usize>() -> ZResult<Self> {
        Self::try_from(Vec::<u8, N>::new())
    }

    /// # Safety
    ///
    /// Buffer modification must not modify slice range.
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        self.buf.as_mut_slice()
    }

    // This method is internal and is only meant to be used in `ZBufWriter`.
    // It's implemented in this module because it plays with `ZSlice` invariant,
    // so it should stay in the same module.
    // See https://github.com/eclipse-zenoh/zenoh/pull/1289#discussion_r1701796640
    #[inline]
    pub(crate) fn writer<const N: usize>(&mut self) -> Option<ZSliceWriter<'_, N>> {
        let vec = self.buf.as_mut_vec::<N>()?;

        if self.end == vec.len() {
            Some(ZSliceWriter {
                vec,
                end: &mut self.end,
            })
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn subslice(&self, range: impl RangeBounds<usize>) -> Option<Self> {
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len(),
        };
        if start <= end && end <= self.len() {
            Some(ZSlice {
                buf: self.buf.clone(),
                start: self.start + start,
                end: self.start + end,
            })
        } else {
            None
        }
    }
}

impl<const N: usize> TryFrom<Vec<u8, N>> for ZSlice {
    type Error = ZError;

    fn try_from(value: Vec<u8, N>) -> ZResult<Self> {
        let len = value.len();
        Ok(Self {
            buf: ABuf::try_from(value)?,
            start: 0,
            end: len,
        })
    }
}

impl Deref for ZSlice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for ZSlice {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<Rhs: AsRef<[u8]> + ?Sized> PartialEq<Rhs> for ZSlice {
    fn eq(&self, other: &Rhs) -> bool {
        self.as_slice() == other.as_ref()
    }
}

impl Eq for ZSlice {}

impl hash::Hash for ZSlice {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl fmt::Display for ZSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for ZSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x?}", self.as_slice())
    }
}

// Buffer
impl Buffer for ZSlice {
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl Buffer for &ZSlice {
    fn len(&self) -> usize {
        ZSlice::len(self)
    }
}

impl Buffer for &mut ZSlice {
    fn len(&self) -> usize {
        ZSlice::len(self)
    }
}

// SplitBuffer
impl SplitBuffer for ZSlice {
    type Slices<'a> = core::iter::Once<&'a [u8]>;

    fn slices(&self) -> Self::Slices<'_> {
        core::iter::once(self.as_slice())
    }
}

#[derive(Debug)]
pub(crate) struct ZSliceWriter<'a, const N: usize> {
    vec: &'a mut Vec<u8, N>,
    end: &'a mut usize,
}

impl<const N: usize> Writer for ZSliceWriter<'_, N> {
    fn write(&mut self, bytes: &[u8]) -> ZResult<NonZeroUsize> {
        let len = self.vec.write(bytes)?;
        *self.end += len.get();
        Ok(len)
    }

    fn write_exact(&mut self, bytes: &[u8]) -> ZResult<()> {
        self.write(bytes).map(|_| ())
    }

    fn remaining(&self) -> usize {
        self.vec.remaining()
    }

    unsafe fn with_slot<F>(&mut self, len: usize, write: F) -> ZResult<NonZeroUsize>
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        // SAFETY: same precondition as the enclosing function
        let len = unsafe { self.vec.with_slot(len, write) }?;
        *self.end += len.get();
        Ok(len)
    }
}

impl<const N: usize> BacktrackableWriter for ZSliceWriter<'_, N> {
    type Mark = usize;

    fn mark(&mut self) -> Self::Mark {
        *self.end
    }

    fn rewind(&mut self, mark: Self::Mark) -> bool {
        assert!(mark <= self.vec.len());
        self.vec.truncate(mark);
        *self.end = mark;
        true
    }
}

// Reader
impl HasReader for &mut ZSlice {
    type Reader = Self;

    fn reader(self) -> Self::Reader {
        self
    }
}

impl Reader for ZSlice {
    fn read(&mut self, into: &mut [u8]) -> ZResult<NonZeroUsize> {
        let mut reader = self.as_slice().reader();
        let len = reader.read(into)?;
        // we trust `Reader` impl for `&[u8]` to not overflow the size of the slice
        self.start += len.get();
        Ok(len)
    }

    fn read_exact(&mut self, into: &mut [u8]) -> ZResult<()> {
        let mut reader = self.as_slice().reader();
        reader.read_exact(into)?;
        // we trust `Reader` impl for `&[u8]` to not overflow the size of the slice
        self.start += into.len();
        Ok(())
    }

    fn remaining(&self) -> usize {
        self.len()
    }

    fn read_zslices<F: FnMut(ZSlice), const N: usize>(&mut self, mut f: F) -> ZResult<()> {
        let zslice = self.read_zslice::<N>()?;
        f(zslice);
        Ok(())
    }

    fn read_zslice<const N: usize>(&mut self) -> ZResult<ZSlice> {
        let res = self.subslice(..N).ok_or(zerr!(ZE::DidntRead))?;
        self.start += N;
        Ok(res)
    }

    fn read_u8(&mut self) -> ZResult<u8> {
        let mut reader = self.as_slice().reader();
        let res = reader.read_u8()?;
        // we trust `Reader` impl for `&[u8]` to not overflow the size of the slice
        self.start += 1;
        Ok(res)
    }

    fn can_read(&self) -> bool {
        !self.is_empty()
    }
}

impl BacktrackableReader for ZSlice {
    type Mark = usize;

    fn mark(&mut self) -> Self::Mark {
        self.start
    }

    fn rewind(&mut self, mark: Self::Mark) -> bool {
        assert!(mark <= self.end);
        self.start = mark;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zslice() {
        // Initialize the pool with capacity 1 for 16-byte buffers
        zunsafe_arc_pool_init!(ArcBytes16: 1);

        let buf = crate::vec::uninit::<16>();
        let mut zslice: ZSlice = buf.clone().try_into().unwrap();
        assert_eq!(buf.as_slice(), zslice.as_slice());

        // SAFETY: buffer slize size is not modified
        let mut_slice = zslice.as_mut_slice().unwrap();

        mut_slice[..buf.len()].clone_from_slice(&buf[..]);

        assert_eq!(buf.as_slice(), zslice.as_slice());
    }
}
