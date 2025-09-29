use core::{
    convert::TryFrom,
    fmt,
    ops::{Deref, Div},
    str::FromStr,
};

use heapless::{format, String};
use zenoh_result::{zerr, ZError, ZResult, ZE};

use crate::key_expr::{canon::Canonize, keyexpr, nonwild_keyexpr, ZKeyLength};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnedKeyExpr<const N: usize>(pub(crate) String<N>);

impl<const N: usize> OwnedKeyExpr<N> {
    pub fn new<T>(t: T) -> ZResult<Self>
    where
        Self: TryFrom<T, Error = ZError>,
    {
        Self::try_from(t)
    }

    pub fn autocanonize<T>(mut t: T) -> ZResult<Self>
    where
        Self: TryFrom<T, Error = ZError>,
        T: Canonize,
    {
        t.canonize();
        Self::new(t)
    }

    pub unsafe fn from_string_unchecked(s: String<N>) -> Self {
        Self::from_str_unchecked(s.as_str())
    }

    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        OwnedKeyExpr(s.try_into().unwrap())
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize, const L: usize> Div<&keyexpr> for (OwnedKeyExpr<N>, ZKeyLength<L>) {
    type Output = OwnedKeyExpr<L>;

    fn div(self, rhs: &keyexpr) -> Self::Output {
        (&self.0, self.1) / rhs
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize, const L: usize> Div<&keyexpr> for (&OwnedKeyExpr<N>, ZKeyLength<L>) {
    type Output = OwnedKeyExpr<L>;

    fn div(self, rhs: &keyexpr) -> Self::Output {
        if N + rhs.as_str().len() + 1 > L {
            panic!(
                "key expression too long, required L > {}",
                N + rhs.as_str().len() + 1
            );
        }

        let s: String<L> = format!("{}/{}", self.0.as_str(), rhs.as_str()).unwrap();

        OwnedKeyExpr::autocanonize(s).unwrap()
    }
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
impl<const N: usize, const L: usize> Div<&keyexpr> for (&OwnedNonWildKeyExpr<N>, ZKeyLength<L>) {
    type Output = OwnedKeyExpr<L>;

    fn div(self, rhs: &keyexpr) -> Self::Output {
        if N + rhs.as_str().len() + 1 > L {
            panic!(
                "key expression too long, required L > {}",
                N + rhs.as_str().len() + 1
            );
        }

        let s: String<L> = format!("{}/{}", self.0.as_str(), rhs.as_str()).unwrap();

        OwnedKeyExpr::autocanonize(s).unwrap()
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<const N: usize, const L: usize> Div<&nonwild_keyexpr>
    for (&OwnedNonWildKeyExpr<N>, ZKeyLength<L>)
{
    type Output = OwnedKeyExpr<L>;

    fn div(self, rhs: &nonwild_keyexpr) -> Self::Output {
        if N + rhs.as_str().len() + 1 > L {
            panic!(
                "key expression too long, required L > {}",
                N + rhs.as_str().len() + 1
            );
        }

        let s: String<L> = format!("{}/{}", self.0.as_str(), rhs.as_str()).unwrap();
        s.try_into().unwrap()
    }
}
