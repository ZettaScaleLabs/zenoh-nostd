use core::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::Deref,
};

use heapless::String;

use crate::{
    keyexpr::{ZKeyError, canon::Canonize},
    result::ZResult,
    zbail,
};

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct keyexpr(str);

impl keyexpr {
    pub(crate) fn new<'a, T>(t: &'a T) -> ZResult<&'a Self, ZKeyError>
    where
        &'a Self: TryFrom<&'a T, Error = ZKeyError>,
        T: ?Sized,
    {
        t.try_into()
    }

    pub(crate) fn autocanonize<'a, T>(t: &'a mut T) -> ZResult<&'a Self, ZKeyError>
    where
        &'a Self: TryFrom<&'a T, Error = ZKeyError>,
        T: Canonize + ?Sized,
    {
        t.canonize();
        Self::new(t)
    }

    pub(crate) fn intersects(&self, other: &Self) -> bool {
        use super::intersect::Intersector;
        super::intersect::DEFAULT_INTERSECTOR.intersect(self, other)
    }

    pub(crate) fn includes(&self, other: &Self) -> bool {
        use super::include::Includer;
        super::include::DEFAULT_INCLUDER.includes(self, other)
    }

    pub(crate) fn is_wild_impl(&self) -> bool {
        self.0.contains(super::SINGLE_WILD as char)
    }

    pub(crate) const fn is_double_wild(&self) -> bool {
        let bytes = self.0.as_bytes();
        bytes.len() == 2 && bytes[0] == b'*'
    }

    pub const fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) const fn from_str_unchecked(s: &str) -> &Self {
        unsafe { core::mem::transmute(s) }
    }

    pub(crate) fn from_slice_unchecked(s: &[u8]) -> &Self {
        unsafe { core::mem::transmute(s) }
    }

    pub(crate) fn first_byte(&self) -> u8 {
        unsafe { *self.as_bytes().get_unchecked(0) }
    }
}

impl fmt::Debug for keyexpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ke`{}`", self.as_ref())
    }
}

impl fmt::Display for keyexpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl<'a> TryFrom<&'a str> for &'a keyexpr {
    type Error = ZKeyError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.is_empty() || value.ends_with('/') {
            zbail!(ZKeyError::KeyExprNotMatch);
        }
        let bytes = value.as_bytes();

        let mut chunk_start = 0;

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                c if c > b'/' && c != b'?' => i += 1,

                b'/' if i == chunk_start => zbail!(ZKeyError::KeyExprNotMatch),

                b'/' => {
                    i += 1;
                    chunk_start = i;
                }

                b'*' if i != chunk_start => zbail!(ZKeyError::KeyExprNotMatch),

                b'*' => match bytes.get(i + 1) {
                    None => break,

                    Some(&b'/') => {
                        i += 2;
                        chunk_start = i;
                    }

                    Some(&b'*') => match bytes.get(i + 2) {
                        None => break,

                        Some(&b'/') if matches!(bytes.get(i + 3), Some(b'*')) => {
                            zbail!(ZKeyError::KeyExprNotMatch)
                        }

                        Some(&b'/') => {
                            i += 3;
                            chunk_start = i;
                        }

                        _ => zbail!(ZKeyError::KeyExprNotMatch),
                    },

                    _ => zbail!(ZKeyError::KeyExprNotMatch),
                },

                b'$' if bytes.get(i + 1) != Some(&b'*') => {
                    zbail!(ZKeyError::KeyExprNotMatch);
                }

                b'$' => match bytes.get(i + 2) {
                    Some(&b'$') => zbail!(ZKeyError::KeyExprNotMatch),

                    Some(&b'/') | None if i == chunk_start => {
                        zbail!(ZKeyError::KeyExprNotMatch)
                    }

                    None => break,

                    _ => i += 2,
                },

                b'#' | b'?' => zbail!(ZKeyError::KeyExprNotMatch),

                _ => i += 1,
            }
        }
        Ok(keyexpr::from_str_unchecked(value))
    }
}

impl<'a> TryFrom<&'a mut str> for &'a keyexpr {
    type Error = ZKeyError;
    fn try_from(value: &'a mut str) -> Result<Self, Self::Error> {
        (value as &'a str).try_into()
    }
}

impl<'a, const N: usize> TryFrom<&'a mut String<N>> for &'a keyexpr {
    type Error = ZKeyError;
    fn try_from(value: &'a mut String<N>) -> Result<Self, Self::Error> {
        (value.as_str()).try_into()
    }
}

impl<'a, const N: usize> TryFrom<&'a String<N>> for &'a keyexpr {
    type Error = ZKeyError;
    fn try_from(value: &'a String<N>) -> Result<Self, Self::Error> {
        (value.as_str()).try_into()
    }
}

impl Deref for keyexpr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}

impl AsRef<str> for keyexpr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl PartialEq<str> for keyexpr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<keyexpr> for str {
    fn eq(&self, other: &keyexpr) -> bool {
        self == other.as_str()
    }
}

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) struct nonwild_keyexpr(keyexpr);

impl nonwild_keyexpr {
    pub(crate) fn new<'a, T, E>(t: &'a T) -> Result<&'a Self, ZKeyError>
    where
        &'a keyexpr: TryFrom<&'a T, Error = E>,
        E: Into<ZKeyError>,
        T: ?Sized,
    {
        let ke: &'a keyexpr = t.try_into().map_err(|e: E| e.into())?;
        ke.try_into()
    }

    /// # Safety
    pub(crate) const fn from_str_unchecked(s: &str) -> &Self {
        unsafe { core::mem::transmute(s) }
    }
}

impl Deref for nonwild_keyexpr {
    type Target = keyexpr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<&'a keyexpr> for &'a nonwild_keyexpr {
    type Error = ZKeyError;
    fn try_from(value: &'a keyexpr) -> Result<Self, Self::Error> {
        if value.is_wild_impl() {
            zbail!(ZKeyError::KeyExprNotMatch);
        }
        Ok(unsafe { core::mem::transmute::<&keyexpr, &nonwild_keyexpr>(value) })
    }
}
