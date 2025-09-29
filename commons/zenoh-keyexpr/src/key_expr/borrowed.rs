use core::{
    borrow::Borrow,
    convert::{TryFrom, TryInto},
    fmt,
    ops::Deref,
};

use heapless::{format, String};
use zenoh_result::{zbail, zerr, ZError, ZResult, ZE};

use super::{canon::Canonize, OwnedKeyExpr, OwnedNonWildKeyExpr};

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct keyexpr(str);

impl keyexpr {
    pub fn new<'a, T>(t: &'a T) -> ZResult<&'a Self>
    where
        &'a Self: TryFrom<&'a T, Error = ZError>,
        T: ?Sized,
    {
        t.try_into()
    }

    pub fn autocanonize<'a, T>(t: &'a mut T) -> ZResult<&'a Self>
    where
        &'a Self: TryFrom<&'a T, Error = ZError>,
        T: Canonize + ?Sized,
    {
        t.canonize();
        Self::new(t)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        use super::intersect::Intersector;
        super::intersect::DEFAULT_INTERSECTOR.intersect(self, other)
    }

    pub fn includes(&self, other: &Self) -> bool {
        use super::include::Includer;
        super::include::DEFAULT_INCLUDER.includes(self, other)
    }

    pub fn join<S: AsRef<str> + ?Sized, const N: usize>(
        &self,
        other: &S,
    ) -> ZResult<OwnedKeyExpr<N>> {
        OwnedKeyExpr::autocanonize(
            format!("{}/{}", self, other.as_ref()).map_err(|_| zerr!(ZE::CapacityExceeded))?,
        )
    }

    pub(crate) fn is_wild_impl(&self) -> bool {
        self.0.contains(super::SINGLE_WILD as char)
    }

    pub const fn is_double_wild(&self) -> bool {
        let bytes = self.0.as_bytes();
        bytes.len() == 2 && bytes[0] == b'*'
    }

    pub const fn as_str(&self) -> &str {
        &self.0
    }

    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }

    pub unsafe fn from_slice_unchecked(s: &[u8]) -> &Self {
        core::mem::transmute(s)
    }

    pub fn first_byte(&self) -> u8 {
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

#[repr(i8)]
enum KeyExprError {
    LoneDollarStar = -1,
    SingleStarAfterDoubleStar = -2,
    DoubleStarAfterDoubleStar = -3,
    EmptyChunk = -4,
    StarInChunk = -5,
    DollarAfterDollar = -6,
    SharpOrQMark = -7,
    UnboundDollar = -8,
}

impl KeyExprError {
    #[cold]
    fn into_err(self, _: &str) -> ZError {
        use zenoh_result::WithContext;

        let error = match &self {
            Self::LoneDollarStar => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: empty chunks are forbidden, as well as leading and trailing slashes"),
            Self::SingleStarAfterDoubleStar => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `**/*` must be replaced by `*/**` to reach canon-form"),
            Self::DoubleStarAfterDoubleStar => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `**/**` must be replaced by `**` to reach canon-form"),
            Self::EmptyChunk => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: empty chunks are forbidden, as well as leading and trailing slashes"),
            Self::StarInChunk => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `*` may only be preceded by `/` or `$`"),
            Self::DollarAfterDollar => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `$` is not allowed after `$*`"),
            Self::SharpOrQMark => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `#` and `?` are forbidden characters"),
            Self::UnboundDollar => zerr!(ZE::KeyExprValidationFailed).ctx("Invalid Key Expr: `$` is only allowed in `$*`")
        };

        error
    }
}

impl<'a> TryFrom<&'a str> for &'a keyexpr {
    type Error = ZError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use KeyExprError::*;
        // Check for emptiness or trailing slash, as they are not caught after.
        if value.is_empty() || value.ends_with('/') {
            return Err(EmptyChunk.into_err(value));
        }
        let bytes = value.as_bytes();
        // The start of the chunk, i.e. the index of the char after the '/'.
        let mut chunk_start = 0;
        // Use a while loop to scan the string because it requires to advance the iteration
        // manually for some characters, e.g. '$'.
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                // In UTF-8, all keyexpr special characters are lesser or equal to '/', except '?'
                // This shortcut greatly reduce the number of operations for alphanumeric
                // characters, which are the most common in keyexprs.
                c if c > b'/' && c != b'?' => i += 1,
                // A chunk cannot start with '/'
                b'/' if i == chunk_start => return Err(EmptyChunk.into_err(value)),
                // `chunk_start` is updated when starting a new chunk.
                b'/' => {
                    i += 1;
                    chunk_start = i;
                }
                // The first encountered '*' must be at the beginning of a chunk
                b'*' if i != chunk_start => return Err(StarInChunk.into_err(value)),
                // When a '*' is match, it means this is a wildcard chunk, possibly with two "**",
                // which must be followed by a '/' or the end of the string.
                // So the next character is checked, and the cursor
                b'*' => match bytes.get(i + 1) {
                    // Break if end of string is reached.
                    None => break,
                    // If a '/' is found, start a new chunk, and advance the cursor to take in
                    // previous check.
                    Some(&b'/') => {
                        i += 2;
                        chunk_start = i;
                    }
                    // If a second '*' is found, the next character must be a slash.
                    Some(&b'*') => match bytes.get(i + 2) {
                        // Break if end of string is reached.
                        None => break,
                        // Because a "**" chunk cannot be followed by "*" or "**", the next char is
                        // checked to not be a '*'.
                        Some(&b'/') if matches!(bytes.get(i + 3), Some(b'*')) => {
                            // If there are two consecutive wildcard chunks, raise the appropriate
                            // error.
                            #[cold]
                            fn double_star_err(value: &str, i: usize) -> ZError {
                                match (value.as_bytes().get(i + 4), value.as_bytes().get(i + 5)) {
                                    (None | Some(&b'/'), _) => SingleStarAfterDoubleStar,
                                    (Some(&b'*'), None | Some(&b'/')) => DoubleStarAfterDoubleStar,
                                    _ => StarInChunk,
                                }
                                .into_err(value)
                            }
                            return Err(double_star_err(value, i));
                        }
                        // If a '/' is found, start a new chunk, and advance the cursor to take in
                        // previous checks.
                        Some(&b'/') => {
                            i += 3;
                            chunk_start = i;
                        }
                        // This is not a "**" chunk, raise an error.
                        _ => return Err(StarInChunk.into_err(value)),
                    },
                    // This is not a "*" chunk, raise an error.
                    _ => return Err(StarInChunk.into_err(value)),
                },
                // A '$' must be followed by '*'.
                b'$' if bytes.get(i + 1) != Some(&b'*') => {
                    return Err(UnboundDollar.into_err(value))
                }
                // "$*" has some additional rules to check.
                b'$' => match bytes.get(i + 2) {
                    // "$*" cannot be followed by '$'.
                    Some(&b'$') => return Err(DollarAfterDollar.into_err(value)),
                    // "$*" cannot be alone in a chunk
                    Some(&b'/') | None if i == chunk_start => {
                        return Err(LoneDollarStar.into_err(value))
                    }
                    // Break if end of string is reached.
                    None => break,
                    // Everything is fine, advance the cursor taking the '*' check in account.
                    _ => i += 2,
                },
                // '#' and '?' are forbidden.
                b'#' | b'?' => return Err(SharpOrQMark.into_err(value)),
                // Fallback for unmatched characters
                _ => i += 1,
            }
        }
        Ok(unsafe { keyexpr::from_str_unchecked(value) })
    }
}

impl<'a> TryFrom<&'a mut str> for &'a keyexpr {
    type Error = ZError;
    fn try_from(value: &'a mut str) -> Result<Self, Self::Error> {
        (value as &'a str).try_into()
    }
}

impl<'a, const N: usize> TryFrom<&'a mut String<N>> for &'a keyexpr {
    type Error = ZError;
    fn try_from(value: &'a mut String<N>) -> Result<Self, Self::Error> {
        (value.as_str()).try_into()
    }
}

impl<'a, const N: usize> TryFrom<&'a String<N>> for &'a keyexpr {
    type Error = ZError;
    fn try_from(value: &'a String<N>) -> Result<Self, Self::Error> {
        (value.as_str()).try_into()
    }
}
impl<'a> TryFrom<&'a &'a str> for &'a keyexpr {
    type Error = ZError;
    fn try_from(value: &'a &'a str) -> Result<Self, Self::Error> {
        (*value).try_into()
    }
}
impl<'a> TryFrom<&'a &'a mut str> for &'a keyexpr {
    type Error = ZError;
    fn try_from(value: &'a &'a mut str) -> Result<Self, Self::Error> {
        keyexpr::new(*value)
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

impl<const N: usize> Borrow<keyexpr> for OwnedKeyExpr<N> {
    fn borrow(&self) -> &keyexpr {
        self
    }
}

/// A keyexpr that is statically known not to contain any wild chunks.
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct nonwild_keyexpr(keyexpr);

impl nonwild_keyexpr {
    pub fn new<'a, T, E>(t: &'a T) -> Result<&'a Self, ZError>
    where
        &'a keyexpr: TryFrom<&'a T, Error = E>,
        E: Into<ZError>,
        T: ?Sized,
    {
        let ke: &'a keyexpr = t.try_into().map_err(|e: E| e.into())?;
        ke.try_into()
    }

    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }
}

impl Deref for nonwild_keyexpr {
    type Target = keyexpr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<&'a keyexpr> for &'a nonwild_keyexpr {
    type Error = ZError;
    fn try_from(value: &'a keyexpr) -> Result<Self, Self::Error> {
        if value.is_wild_impl() {
            zbail!(ZE::WildExprContainsInvalidChunks);
        }
        Ok(unsafe { core::mem::transmute::<&keyexpr, &nonwild_keyexpr>(value) })
    }
}

impl<const N: usize> Borrow<nonwild_keyexpr> for OwnedNonWildKeyExpr<N> {
    fn borrow(&self) -> &nonwild_keyexpr {
        self
    }
}
