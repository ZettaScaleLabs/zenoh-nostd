use core::{
    convert::TryFrom,
    fmt::{self, Debug},
};

use zenoh_buffer::ZBuf;
use zenoh_result::{zbail, zerr, ZError, ZResult, ZE};

/// # Zenoh extensions
///
/// A zenoh extension is encoded as TLV (Type, Length, Value).
/// Zenoh extensions with unknown IDs (i.e., type) can be skipped by reading the length and
/// not decoding the body (i.e. value). In case the zenoh extension is unknown, it is
/// still possible to forward it to the next hops, which in turn may be able to understand it.
/// This results in the capability of introducing new extensions in an already running system
/// without requiring the redeployment of the totality of infrastructure nodes.
///
/// The zenoh extension wire format is the following:
///
/// ```text
/// Header flags:
/// - E |: Encoding     The encoding of the extension
/// - E/
/// - Z: More           If Z==1 then another extension will follow.
///
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// |Z|ENC|M|   ID  |
/// +-+---+-+-------+
/// %    length     % -- If ENC == Z64 || ENC == ZBuf (z32)
/// +---------------+
/// ~     [u8]      ~ -- If ENC == ZBuf
/// +---------------+
///
/// Encoding:
/// - 0b00: Unit
/// - 0b01: Z64
/// - 0b10: ZBuf
/// - 0b11: Reserved
///
/// (*) If the zenoh extension is not understood, then it SHOULD NOT be dropped and it
///     SHOULD be forwarded to the next hops.
/// ```
///
pub mod iext {
    use core::fmt;

    pub const ID_BITS: u8 = 4;
    pub const ID_MASK: u8 = !(u8::MAX << ID_BITS);

    pub const FLAG_M: u8 = 1 << 4;
    pub const ENC_UNIT: u8 = 0b00 << 5;
    pub const ENC_Z64: u8 = 0b01 << 5;
    pub const ENC_ZBUF: u8 = 0b10 << 5;
    pub const ENC_MASK: u8 = 0b11 << 5;
    pub const FLAG_Z: u8 = 1 << 7;

    pub const fn eid(header: u8) -> u8 {
        header & !FLAG_Z
    }

    pub const fn mid(header: u8) -> u8 {
        header & ID_MASK
    }

    pub(super) const fn id(id: u8, mandatory: bool, encoding: u8) -> u8 {
        let mut id = id & ID_MASK;
        if mandatory {
            id |= FLAG_M;
        } else {
            id &= !FLAG_M;
        }
        id |= encoding;
        id
    }

    pub(super) const fn is_mandatory(id: u8) -> bool {
        crate::common::imsg::has_flag(id, FLAG_M)
    }

    pub(super) fn fmt(f: &mut fmt::DebugStruct, id: u8) {
        f.field("Id", &(id & ID_MASK))
            .field("Mandatory", &is_mandatory(id))
            .field(
                "Encoding",
                match id & ENC_MASK {
                    ENC_UNIT => &"Unit",
                    ENC_Z64 => &"Z64",
                    ENC_ZBUF => &"ZBuf",
                    _ => &"Unknown",
                },
            );
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ZExtUnit<const ID: u8>;

impl<const ID: u8> Default for ZExtUnit<{ ID }> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const ID: u8> ZExtUnit<{ ID }> {
    pub const ID: u8 = ID;

    pub const fn new() -> Self {
        Self
    }

    pub const fn id(mandatory: bool) -> u8 {
        iext::id(ID, mandatory, iext::ENC_UNIT)
    }

    pub const fn is_mandatory(&self) -> bool {
        iext::is_mandatory(ID)
    }

    pub const fn transmute<const DI: u8>(self) -> ZExtUnit<{ DI }> {
        ZExtUnit::new()
    }
}

impl<const ID: u8> Debug for ZExtUnit<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtUnit");
        iext::fmt(&mut s, ID);
        s.finish()
    }
}

impl<'a, const ID: u8> TryFrom<ZExtUnknown<'a>> for ZExtUnit<{ ID }> {
    type Error = ZError;

    fn try_from(v: ZExtUnknown<'a>) -> Result<Self, Self::Error> {
        if v.id != ID {
            return Err(zerr!(ZE::ConversionFailure));
        }
        match v.body {
            ZExtBody::Unit => Ok(Self::new()),
            _ => Err(zerr!(ZE::ConversionFailure)),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ZExtZ64<const ID: u8> {
    pub value: u64,
}

impl<const ID: u8> ZExtZ64<{ ID }> {
    pub const ID: u8 = ID;

    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    pub const fn id(mandatory: bool) -> u8 {
        iext::id(ID, mandatory, iext::ENC_Z64)
    }

    pub const fn is_mandatory(&self) -> bool {
        iext::is_mandatory(ID)
    }

    pub const fn transmute<const DI: u8>(self) -> ZExtZ64<{ DI }> {
        ZExtZ64::new(self.value)
    }
}

impl<const ID: u8> Debug for ZExtZ64<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZ64");
        iext::fmt(&mut s, ID);
        s.field("Value", &self.value).finish()
    }
}

impl<'a, const ID: u8> TryFrom<ZExtUnknown<'a>> for ZExtZ64<{ ID }> {
    type Error = ZError;

    fn try_from(v: ZExtUnknown<'a>) -> Result<Self, Self::Error> {
        if v.id != ID {
            return Err(zerr!(ZE::ConversionFailure));
        }
        match v.body {
            ZExtBody::Z64(v) => Ok(Self::new(v)),
            _ => Err(zerr!(ZE::ConversionFailure)),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
pub struct ZExtZBuf<'a, const ID: u8> {
    pub value: ZBuf<'a>,
}

impl<'a, const ID: u8> ZExtZBuf<'a, { ID }> {
    pub const ID: u8 = ID;

    pub const fn new(value: ZBuf<'a>) -> Self {
        Self { value }
    }

    pub const fn id(mandatory: bool) -> u8 {
        iext::id(ID, mandatory, iext::ENC_ZBUF)
    }

    pub const fn is_mandatory(&self) -> bool {
        iext::is_mandatory(ID)
    }

    pub fn transmute<const DI: u8>(self) -> ZExtZBuf<'a, { DI }> {
        ZExtZBuf::new(self.value)
    }
}

impl<const ID: u8> Debug for ZExtZBuf<'_, { ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZBuf");
        iext::fmt(&mut s, ID);
        s.field("Value", &self.value).finish()
    }
}

impl<'a, const ID: u8> TryFrom<ZExtUnknown<'a>> for ZExtZBuf<'a, { ID }> {
    type Error = ZError;

    fn try_from(v: ZExtUnknown<'a>) -> ZResult<Self> {
        if v.id != ID {
            zbail!(ZE::ConversionFailure);
        }
        match v.body {
            ZExtBody::ZBuf(v) => Ok(Self::new(v)),
            _ => Err(zerr!(ZE::ConversionFailure)),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ZExtZBufHeader<const ID: u8> {
    pub len: usize,
}

impl<const ID: u8> ZExtZBufHeader<{ ID }> {
    pub const ID: u8 = ID;

    pub const fn new(len: usize) -> Self {
        Self { len }
    }

    pub const fn id(mandatory: bool) -> u8 {
        iext::id(ID, mandatory, iext::ENC_ZBUF)
    }

    pub const fn is_mandatory(&self) -> bool {
        iext::is_mandatory(ID)
    }
}

impl<const ID: u8> Debug for ZExtZBufHeader<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZBufHeader");
        iext::fmt(&mut s, ID);
        s.field("Len", &self.len).finish()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ZExtBody<'a> {
    #[default]
    Unit,
    Z64(u64),
    ZBuf(ZBuf<'a>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct ZExtUnknown<'a> {
    pub id: u8,
    pub body: ZExtBody<'a>,
}

impl<'a> ZExtUnknown<'a> {
    pub const fn new(id: u8, mandatory: bool, body: ZExtBody<'a>) -> Self {
        let enc = match &body {
            ZExtBody::Unit => iext::ENC_UNIT,
            ZExtBody::Z64(_) => iext::ENC_Z64,
            ZExtBody::ZBuf(_) => iext::ENC_ZBUF,
        };
        let id = iext::id(id, mandatory, enc);
        Self { id, body }
    }

    pub const fn is_mandatory(&self) -> bool {
        iext::is_mandatory(self.id)
    }
}

impl Debug for ZExtUnknown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtUnknown");
        iext::fmt(&mut s, self.id);
        match &self.body {
            ZExtBody::Unit => {}
            ZExtBody::Z64(v) => {
                s.field("Value", v);
            }
            ZExtBody::ZBuf(v) => {
                s.field("Value", v);
            }
        };
        s.finish()
    }
}

impl<const ID: u8> From<ZExtUnit<{ ID }>> for ZExtUnknown<'_> {
    fn from(_: ZExtUnit<{ ID }>) -> Self {
        ZExtUnknown {
            id: ID,
            body: ZExtBody::Unit,
        }
    }
}

impl<const ID: u8> From<ZExtZ64<{ ID }>> for ZExtUnknown<'_> {
    fn from(e: ZExtZ64<{ ID }>) -> Self {
        ZExtUnknown {
            id: ID,
            body: ZExtBody::Z64(e.value),
        }
    }
}

impl<'a, const ID: u8> From<ZExtZBuf<'a, { ID }>> for ZExtUnknown<'a> {
    fn from(e: ZExtZBuf<'a, { ID }>) -> Self {
        ZExtUnknown {
            id: ID,
            body: ZExtBody::ZBuf(e.value),
        }
    }
}

// Macros
#[macro_export]
macro_rules! zextunit {
    ($id:expr, $m:expr) => {
        $crate::common::extension::ZExtUnit<{ $crate::common::extension::ZExtUnit::<$id>::id($m) }>
    }
}

#[macro_export]
macro_rules! zextz64 {
    ($id:expr, $m:expr) => {
        $crate::common::extension::ZExtZ64<{ $crate::common::extension::ZExtZ64::<$id>::id($m) }>
    }
}

#[macro_export]
macro_rules! zextzbuf {
    ($lt:lifetime, $id:expr, $m:expr) => {
        $crate::common::extension::ZExtZBuf<$lt, { $crate::common::extension::ZExtZBuf::<$id>::id($m) }>
    }
}
