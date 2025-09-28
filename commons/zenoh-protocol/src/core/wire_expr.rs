use core::{convert::TryInto, fmt, sync::atomic::AtomicU16};

use heapless::String;
use zenoh_keyexpr::key_expr::{keyexpr, OwnedKeyExpr};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::{core::CowStr, network::Mapping};

/// A numerical Id mapped to a key expression.
pub type ExprId = u16;
pub type ExprLen = u16;

pub type AtomicExprId = AtomicU16;
pub const EMPTY_EXPR_ID: ExprId = 0;

/// A zenoh **resource** is represented by a pair composed by a **key** and a
/// **value**, such as, ```(car/telemetry/speed, 320)```.  A **resource key**
/// is an arbitrary array of characters, with the exclusion of the symbols
/// ```*```, ```**```, ```?```, ```[```, ```]```, and ```#```,
/// which have special meaning in the context of zenoh.
///
/// A key including any number of the wildcard symbols, ```*``` and ```**```,
/// such as, ```/car/telemetry/*```, is called a **key expression** as it
/// denotes a set of keys. The wildcard character ```*``` expands to an
/// arbitrary string not including zenoh's reserved characters and the ```/```
/// character, while the ```**``` expands to  strings that may also include the
/// ```/``` character.
///
/// Finally, it is worth mentioning that for time and space efficiency matters,
/// zenoh will automatically map key expressions to small integers. The mapping is automatic,
/// but it can be triggered excplicily by with `zenoh::Session::declare_keyexpr()`.
///
//
//  7 6 5 4 3 2 1 0
// +-+-+-+-+-+-+-+-+
// ~      id       — if Expr: id=0
// +-+-+-+-+-+-+-+-+
// ~    suffix     ~ if flag K==1 in Message's header
// +---------------+
//
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct WireExpr<'a, const N: usize> {
    pub scope: ExprId, // 0 marks global scope
    pub suffix: CowStr<'a, N>,
    pub mapping: Mapping,
}

impl<'a, const N: usize> WireExpr<'a, N> {
    pub fn empty() -> Self {
        WireExpr {
            scope: 0,
            suffix: CowStr::Borrowed(""),
            mapping: Mapping::Sender,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.scope == 0 && self.suffix.as_ref().is_empty()
    }

    pub fn as_str(&'a self) -> &'a str {
        if self.scope == 0 {
            self.suffix.as_ref()
        } else {
            "<encoded_expr>"
        }
    }

    pub fn try_as_str(&'a self) -> ZResult<&'a str> {
        if self.scope == EMPTY_EXPR_ID {
            Ok(self.suffix.as_ref())
        } else {
            bail!(ZE::ScopedKeyExpr)
        }
    }

    pub fn as_id(&'a self) -> ExprId {
        self.scope
    }

    pub fn try_as_id(&'a self) -> ZResult<ExprId> {
        if self.has_suffix() {
            bail!(ZE::SuffixedKeyExpr)
        } else {
            Ok(self.scope)
        }
    }

    pub fn as_id_and_suffix(&'a self) -> (ExprId, &'a str) {
        (self.scope, self.suffix.as_ref())
    }

    pub fn has_suffix(&self) -> bool {
        !self.suffix.as_ref().is_empty()
    }

    pub fn to_owned(&self) -> ZResult<WireExpr<'static, N>> {
        Ok(WireExpr {
            scope: self.scope,
            suffix: self.suffix.to_owned()?,
            mapping: self.mapping,
        })
    }

    pub fn with_suffix(mut self, suffix: &'a str) -> ZResult<Self> {
        if suffix.len() + self.suffix.as_ref().len() > N {
            bail!(ZE::CapacityExceeded);
        }

        if self.suffix.as_ref().is_empty() {
            self.suffix = CowStr::Borrowed(suffix);
        } else {
            self.suffix = CowStr::Owned({
                let mut owned = String::<N>::new();
                owned
                    .push_str(self.suffix.as_ref())
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                owned
                    .push_str(suffix)
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                owned
            });
        }
        Ok(self)
    }
}

impl<const N: usize> TryInto<String<N>> for WireExpr<'_, N> {
    type Error = ZError;
    fn try_into(self) -> Result<String<N>, Self::Error> {
        if self.scope == 0 {
            Ok(self.suffix.into_owned()?)
        } else {
            bail!(ZE::ScopedKeyExpr)
        }
    }
}

impl<const N: usize> TryInto<ExprId> for WireExpr<'_, N> {
    type Error = ZError;
    fn try_into(self) -> Result<ExprId, Self::Error> {
        self.try_as_id()
    }
}

impl<const N: usize> From<ExprId> for WireExpr<'_, N> {
    fn from(scope: ExprId) -> Self {
        Self {
            scope,
            suffix: CowStr::Borrowed(""),
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a OwnedKeyExpr<N>> for WireExpr<'a, N> {
    fn from(val: &'a OwnedKeyExpr<N>) -> Self {
        WireExpr {
            scope: 0,
            suffix: CowStr::Borrowed(val.as_str()),
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a keyexpr> for WireExpr<'a, N> {
    fn from(val: &'a keyexpr) -> Self {
        WireExpr {
            scope: 0,
            suffix: CowStr::Borrowed(val.as_str()),
            mapping: Mapping::Sender,
        }
    }
}

impl<const N: usize> fmt::Display for WireExpr<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.scope == 0 {
            write!(f, "{}", self.suffix)
        } else {
            write!(f, "{}:{:?}:{}", self.scope, self.mapping, self.suffix)
        }
    }
}

impl<'a, const N: usize> From<&WireExpr<'a, N>> for WireExpr<'a, N> {
    #[inline]
    fn from(key: &WireExpr<'a, N>) -> WireExpr<'a, N> {
        key.clone()
    }
}

impl<'a, const N: usize> From<&'a str> for WireExpr<'a, N> {
    #[inline]
    fn from(name: &'a str) -> WireExpr<'a, N> {
        WireExpr {
            scope: 0,
            suffix: CowStr::Borrowed(name),
            mapping: Mapping::Sender,
        }
    }
}

impl<const N: usize> From<String<N>> for WireExpr<'_, N> {
    #[inline]
    fn from(name: String<N>) -> WireExpr<'static, N> {
        WireExpr {
            scope: 0,
            suffix: CowStr::Owned(name),
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a String<N>> for WireExpr<'a, N> {
    #[inline]
    fn from(name: &'a String<N>) -> WireExpr<'a, N> {
        WireExpr {
            scope: 0,
            suffix: CowStr::Borrowed(name.as_str()),
            mapping: Mapping::Sender,
        }
    }
}
