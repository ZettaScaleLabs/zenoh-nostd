use core::{convert::TryInto, fmt, sync::atomic::AtomicU16};

use heapless::String;
use zenoh_keyexpr::key_expr::{keyexpr, OwnedKeyExpr};
use zenoh_result::{zbail, ZError, ZResult, ZE};

use crate::network::Mapping;

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
pub struct WireExpr<'a> {
    pub scope: ExprId, // 0 marks global scope
    pub suffix: &'a str,
    pub mapping: Mapping,
}

impl<'a> WireExpr<'a> {
    pub fn new(scope: ExprId, suffix: &'a str, mapping: Mapping) -> Self {
        WireExpr {
            scope,
            suffix,
            mapping,
        }
    }

    pub fn empty() -> Self {
        WireExpr {
            scope: 0,
            suffix: "",
            mapping: Mapping::Sender,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.scope == 0 && self.suffix.is_empty()
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
            zbail!(ZE::ScopedKeyExprUnsupported)
        }
    }

    pub fn as_id(&'a self) -> ExprId {
        self.scope
    }

    pub fn try_as_id(&'a self) -> ZResult<ExprId> {
        if self.has_suffix() {
            zbail!(ZE::SuffixedKeyExprUnsupported);
        } else {
            Ok(self.scope)
        }
    }

    pub fn as_id_and_suffix(&'a self) -> (ExprId, &'a str) {
        (self.scope, self.suffix.as_ref())
    }

    pub fn has_suffix(&self) -> bool {
        !self.suffix.is_empty()
    }

    pub fn with_suffix(&self, suffix: &'a str) -> Self {
        WireExpr {
            scope: self.scope,
            suffix,
            mapping: self.mapping,
        }
    }
}

impl TryInto<ExprId> for WireExpr<'_> {
    type Error = ZError;
    fn try_into(self) -> Result<ExprId, Self::Error> {
        self.try_as_id()
    }
}

impl From<ExprId> for WireExpr<'_> {
    fn from(scope: ExprId) -> Self {
        Self {
            scope,
            suffix: "",
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a OwnedKeyExpr<N>> for WireExpr<'a> {
    fn from(val: &'a OwnedKeyExpr<N>) -> Self {
        WireExpr {
            scope: 0,
            suffix: val.as_str(),
            mapping: Mapping::Sender,
        }
    }
}

impl<'a> From<&'a keyexpr> for WireExpr<'a> {
    fn from(val: &'a keyexpr) -> Self {
        WireExpr {
            scope: 0,
            suffix: val.as_str(),
            mapping: Mapping::Sender,
        }
    }
}

impl fmt::Display for WireExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.scope == 0 {
            write!(f, "{}", self.suffix)
        } else {
            write!(f, "{}:{:?}:{}", self.scope, self.mapping, self.suffix)
        }
    }
}

impl<'a> From<&WireExpr<'a>> for WireExpr<'a> {
    #[inline]
    fn from(key: &WireExpr<'a>) -> WireExpr<'a> {
        key.clone()
    }
}

impl<'a> From<&'a str> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a str) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name,
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a String<N>> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a String<N>) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name.as_str(),
            mapping: Mapping::Sender,
        }
    }
}
