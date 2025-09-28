use core::{
    convert::TryFrom,
    fmt,
    ops::{Deref, Div},
    str::FromStr,
};

use heapless::{format, String};
use zenoh_result::{zerr, ZError, ZResult, ZE};

use crate::key_expr::{canon::Canonize, keyexpr, nonwild_keyexpr};

/// A [`Arc<str>`] newtype that is statically known to be a valid key expression.
///
/// See [`keyexpr`](super::borrowed::keyexpr).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnedKeyExpr<const N: usize>(pub(crate) String<N>);

impl<const N: usize> OwnedKeyExpr<N> {
    /// Equivalent to `<OwnedKeyExpr as TryFrom>::try_from(t)`.
    ///
    /// Will return an Err if `t` isn't a valid key expression.
    /// Note that to be considered a valid key expression, a string MUST be canon.
    ///
    /// [`OwnedKeyExpr::autocanonize`] is an alternative constructor that will canonize the passed expression before constructing it.
    pub fn new<T, E>(t: T) -> Result<Self, E>
    where
        Self: TryFrom<T, Error = E>,
    {
        Self::try_from(t)
    }

    /// Canonizes the passed value before returning it as an `OwnedKeyExpr`.
    ///
    /// Will return Err if the passed value isn't a valid key expression despite canonization.
    pub fn autocanonize<T, E>(mut t: T) -> Result<Self, E>
    where
        Self: TryFrom<T, Error = E>,
        T: Canonize,
    {
        t.canonize();
        Self::new(t)
    }

    /// Constructs an OwnedKeyExpr without checking [`keyexpr`]'s invariants
    /// # Safety
    /// Key Expressions must follow some rules to be accepted by a Zenoh network.
    /// Messages addressed with invalid key expressions will be dropped.
    pub unsafe fn from_string_unchecked(s: String<N>) -> Self {
        Self::from_boxed_str_unchecked(s.as_str())
    }
    /// Constructs an OwnedKeyExpr without checking [`keyexpr`]'s invariants
    /// # Safety
    /// Key Expressions must follow some rules to be accepted by a Zenoh network.
    /// Messages addressed with invalid key expressions will be dropped.
    pub unsafe fn from_boxed_str_unchecked(s: &str) -> Self {
        OwnedKeyExpr(s.try_into().unwrap())
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize> Div<&keyexpr> for OwnedKeyExpr<N> {
    type Output = Self;
    fn div(self, rhs: &keyexpr) -> Self::Output {
        &self / rhs
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize> Div<&keyexpr> for &OwnedKeyExpr<N> {
    type Output = OwnedKeyExpr<N>;
    fn div(self, rhs: &keyexpr) -> Self::Output {
        let s: String<N> =
            format!("{}/{}", self.as_str(), rhs.as_str()).expect("key expression too long");
        OwnedKeyExpr::autocanonize(s).unwrap() // Joining 2 key expressions should always result in a canonizable string.
    }
}

#[test]
fn div() {
    let a = OwnedKeyExpr::<3>::new("a").unwrap();
    let b = OwnedKeyExpr::<1>::new("b").unwrap();
    let k = a / &b;
    assert_eq!(k.as_str(), "a/b")
}

impl<const N: usize> fmt::Debug for OwnedKeyExpr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<const N: usize> fmt::Display for OwnedKeyExpr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<const N: usize> Deref for OwnedKeyExpr<N> {
    type Target = keyexpr;
    fn deref(&self) -> &Self::Target {
        unsafe { keyexpr::from_str_unchecked(&self.0) }
    }
}

impl<const N: usize> AsRef<str> for OwnedKeyExpr<N> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> FromStr for OwnedKeyExpr<N> {
    type Err = ZError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))?)
    }
}

impl<const N: usize> TryFrom<String<N>> for OwnedKeyExpr<N> {
    type Error = ZError;
    fn try_from(value: String<N>) -> Result<Self, Self::Error> {
        <&keyexpr as TryFrom<&str>>::try_from(value.as_str())?;
        Ok(Self(value.into()))
    }
}

impl<'a, const N: usize> TryFrom<&'a keyexpr> for OwnedKeyExpr<N> {
    type Error = ZError;

    fn try_from(val: &'a keyexpr) -> ZResult<Self> {
        OwnedKeyExpr::from_str(val.as_str())
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for OwnedKeyExpr<N> {
    type Error = ZError;

    fn try_from(val: &'a str) -> ZResult<Self> {
        OwnedKeyExpr::from_str(val)
    }
}

impl<const N: usize> From<OwnedKeyExpr<N>> for String<N> {
    fn from(ke: OwnedKeyExpr<N>) -> Self {
        ke.0
    }
}

/// A [`Arc<str>`] newtype that is statically known to be a valid nonwild key expression.
///
/// See [`nonwild_keyexpr`](super::borrowed::nonwild_keyexpr).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnedNonWildKeyExpr<const N: usize>(pub(crate) String<N>);

impl<const N: usize> TryFrom<String<N>> for OwnedNonWildKeyExpr<N> {
    type Error = ZError;
    fn try_from(value: String<N>) -> Result<Self, Self::Error> {
        let ke = <&keyexpr as TryFrom<&str>>::try_from(value.as_str())?;
        <&nonwild_keyexpr as TryFrom<&keyexpr>>::try_from(ke)?;
        Ok(Self(value.into()))
    }
}
impl<'a, const N: usize> TryFrom<&'a nonwild_keyexpr> for OwnedNonWildKeyExpr<N> {
    type Error = ZError;

    fn try_from(val: &'a nonwild_keyexpr) -> ZResult<Self> {
        Ok(OwnedNonWildKeyExpr(
            String::from_str(val.as_str()).map_err(|_| zerr!(ZE::CapacityExceeded))?,
        ))
    }
}

impl<const N: usize> Deref for OwnedNonWildKeyExpr<N> {
    type Target = nonwild_keyexpr;
    fn deref(&self) -> &Self::Target {
        unsafe { nonwild_keyexpr::from_str_unchecked(&self.0) }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize> Div<&keyexpr> for &OwnedNonWildKeyExpr<N> {
    type Output = OwnedKeyExpr<N>;
    fn div(self, rhs: &keyexpr) -> Self::Output {
        let s: String<N> =
            format!("{}/{}", self.as_str(), rhs.as_str()).expect("key expression too long");
        OwnedKeyExpr::autocanonize(s).unwrap() // Joining 2 key expressions should always result in a canonizable string.
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize> Div<&nonwild_keyexpr> for &OwnedNonWildKeyExpr<N> {
    type Output = OwnedKeyExpr<N>;
    fn div(self, rhs: &nonwild_keyexpr) -> Self::Output {
        let s: String<N> =
            format!("{}/{}", self.as_str(), rhs.as_str()).expect("key expression too long");
        s.try_into().unwrap() // Joining 2 non wild key expressions should always result in a non wild string.
    }
}
