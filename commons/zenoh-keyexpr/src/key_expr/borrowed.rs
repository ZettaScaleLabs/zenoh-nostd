use core::{
    borrow::Borrow,
    convert::{TryFrom, TryInto},
    fmt,
    ops::Deref,
};

use heapless::{format, String};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use super::{canon::Canonize, OwnedKeyExpr, OwnedNonWildKeyExpr};

/// A [`str`] newtype that is statically known to be a valid key expression.
///
/// The exact key expression specification can be found [here](https://github.com/eclipse-zenoh/roadmap/blob/main/rfcs/ALL/Key%20Expressions.md). Here are the major lines:
/// * Key expressions are conceptually a `/`-separated list of UTF-8 string typed chunks. These chunks are not allowed to be empty.
/// * Key expressions must be valid UTF-8 strings.
///   Be aware that Zenoh does not perform UTF normalization for you, so get familiar with that concept if your key expression contains glyphs that may have several unicode representation, such as accented characters.
/// * Key expressions may never start or end with `'/'`, nor contain `"//"` or any of the following characters: `#$?`
/// * Key expression must be in canon-form (this ensure that key expressions representing the same set are always the same string).
///   Note that safe constructors will perform canonization for you if this can be done without extraneous allocations.
///
/// Since Key Expressions define sets of keys, you may want to be aware of the hierarchy of [relations](keyexpr::relation_to) between such sets:
/// * Trivially, two sets can have no elements in common: `a/**` and `b/**` for example define two disjoint sets of keys.
/// * Two sets [intersect](keyexpr::intersects()) if they have at least one element in common. `a/*` intersects `*/a` on `a/a` for example.
/// * One set A [includes](keyexpr::includes()) the other set B if all of B's elements are in A: `a/*/**` includes `a/b/**`
/// * Two sets A and B are equal if all A includes B and B includes A. The Key Expression language is designed so that string equality is equivalent to set equality.
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct keyexpr(str);

impl keyexpr {
    /// Equivalent to `<&keyexpr as TryFrom>::try_from(t)`.
    ///
    /// Will return an Err if `t` isn't a valid key expression.
    /// Note that to be considered a valid key expression, a string MUST be canon.
    ///
    /// [`keyexpr::autocanonize`] is an alternative constructor that will canonize the passed expression before constructing it.
    pub fn new<'a, T, E>(t: &'a T) -> Result<&'a Self, E>
    where
        &'a Self: TryFrom<&'a T, Error = E>,
        T: ?Sized,
    {
        t.try_into()
    }

    /// Canonizes the passed value before returning it as a `&keyexpr`.
    ///
    /// Will return Err if the passed value isn't a valid key expression despite canonization.
    ///
    /// Note that this function does not allocate, and will instead mutate the passed value in place during canonization.
    pub fn autocanonize<'a, T, E>(t: &'a mut T) -> Result<&'a Self, E>
    where
        &'a Self: TryFrom<&'a T, Error = E>,
        T: Canonize + ?Sized,
    {
        t.canonize();
        Self::new(t)
    }

    /// Returns `true` if the `keyexpr`s intersect, i.e. there exists at least one key which is contained in both of the sets defined by `self` and `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        use super::intersect::Intersector;
        super::intersect::DEFAULT_INTERSECTOR.intersect(self, other)
    }

    /// Returns `true` if `self` includes `other`, i.e. the set defined by `self` contains every key belonging to the set defined by `other`.
    pub fn includes(&self, other: &Self) -> bool {
        use super::include::Includer;
        super::include::DEFAULT_INCLUDER.includes(self, other)
    }

    /// Joins both sides, inserting a `/` in between them.
    ///
    /// This should be your preferred method when concatenating path segments.
    ///
    /// If `other` is of type `&keyexpr`, you may use `self / other` instead, as the joining becomes infallible.
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

    /// # Safety
    /// This constructs a [`keyexpr`] without ensuring that it is a valid key-expression.
    ///
    /// Much like [`core::str::from_utf8_unchecked`], this is memory-safe, but calling this without maintaining
    /// [`keyexpr`]'s invariants yourself may lead to unexpected behaviors, the Zenoh network dropping your messages.
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }

    /// # Safety
    /// This constructs a [`keyexpr`] without ensuring that it is a valid key-expression.
    ///
    /// Much like [`core::str::from_utf8_unchecked`], this is memory-safe, but calling this without maintaining
    /// [`keyexpr`]'s invariants yourself may lead to unexpected behaviors, the Zenoh network dropping your messages.
    pub unsafe fn from_slice_unchecked(s: &[u8]) -> &Self {
        core::mem::transmute(s)
    }

    pub const fn chunks_impl(&self) -> Chunks<'_> {
        Chunks {
            inner: self.as_str(),
        }
    }
    pub(crate) fn next_delimiter(&self, i: usize) -> Option<usize> {
        self.as_str()
            .get(i + 1..)
            .and_then(|s| s.find('/').map(|j| i + 1 + j))
    }
    pub(crate) fn previous_delimiter(&self, i: usize) -> Option<usize> {
        self.as_str().get(..i).and_then(|s| s.rfind('/'))
    }
    pub fn first_byte(&self) -> u8 {
        unsafe { *self.as_bytes().get_unchecked(0) }
    }
    pub fn iter_splits_ltr(&self) -> SplitsLeftToRight<'_> {
        SplitsLeftToRight {
            inner: self,
            index: 0,
        }
    }
    pub fn iter_splits_rtl(&self) -> SplitsRightToLeft<'_> {
        SplitsRightToLeft {
            inner: self,
            index: self.len(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SplitsLeftToRight<'a> {
    inner: &'a keyexpr,
    index: usize,
}
impl<'a> SplitsLeftToRight<'a> {
    fn right(&self) -> &'a str {
        &self.inner[self.index + ((self.index != 0) as usize)..]
    }
    fn left(&self, followed_by_double: bool) -> &'a str {
        &self.inner[..(self.index + ((self.index != 0) as usize + 2) * followed_by_double as usize)]
    }
}
impl<'a> Iterator for SplitsLeftToRight<'a> {
    type Item = (&'a keyexpr, &'a keyexpr);
    fn next(&mut self) -> Option<Self::Item> {
        match self.index < self.inner.len() {
            false => None,
            true => {
                let right = self.right();
                let double_wild = right.starts_with("**");
                let left = self.left(double_wild);
                self.index = if left.is_empty() {
                    self.inner.next_delimiter(0).unwrap_or(self.inner.len())
                } else {
                    self.inner
                        .next_delimiter(left.len())
                        .unwrap_or(self.inner.len() + (left.len() == self.inner.len()) as usize)
                };
                if left.is_empty() {
                    self.next()
                } else {
                    // SAFETY: because any keyexpr split at `/` becomes 2 valid keyexprs by design, it's safe to assume the constraint is valid once both sides have been validated to not be empty.
                    (!right.is_empty()).then(|| unsafe {
                        (
                            keyexpr::from_str_unchecked(left),
                            keyexpr::from_str_unchecked(right),
                        )
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SplitsRightToLeft<'a> {
    inner: &'a keyexpr,
    index: usize,
}

impl<'a> SplitsRightToLeft<'a> {
    fn right(&self, followed_by_double: bool) -> &'a str {
        &self.inner[(self.index
            - ((self.index != self.inner.len()) as usize + 2) * followed_by_double as usize)..]
    }
    fn left(&self) -> &'a str {
        &self.inner[..(self.index - ((self.index != self.inner.len()) as usize))]
    }
}

impl<'a> Iterator for SplitsRightToLeft<'a> {
    type Item = (&'a keyexpr, &'a keyexpr);
    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            0 => None,
            _ => {
                let left = self.left();
                let double_wild = left.ends_with("**");
                let right = self.right(double_wild);
                self.index = if right.is_empty() {
                    self.inner
                        .previous_delimiter(self.inner.len())
                        .map_or(0, |n| n + 1)
                } else {
                    self.inner
                        .previous_delimiter(
                            self.inner.len()
                                - right.len()
                                - (self.inner.len() != right.len()) as usize,
                        )
                        .map_or(0, |n| n + 1)
                };
                if right.is_empty() {
                    self.next()
                } else {
                    // SAFETY: because any keyexpr split at `/` becomes 2 valid keyexprs by design, it's safe to assume the constraint is valid once both sides have been validated to not be empty.
                    (!left.is_empty()).then(|| unsafe {
                        (
                            keyexpr::from_str_unchecked(left),
                            keyexpr::from_str_unchecked(right),
                        )
                    })
                }
            }
        }
    }
}

#[test]
fn splits() {
    let ke = keyexpr::new("a/**/b/c").unwrap();
    let mut splits = ke.iter_splits_ltr();
    assert_eq!(
        splits.next(),
        Some((
            keyexpr::new("a/**").unwrap(),
            keyexpr::new("**/b/c").unwrap()
        ))
    );
    assert_eq!(
        splits.next(),
        Some((keyexpr::new("a/**/b").unwrap(), keyexpr::new("c").unwrap()))
    );
    assert_eq!(splits.next(), None);
    let mut splits = ke.iter_splits_rtl();
    assert_eq!(
        splits.next(),
        Some((keyexpr::new("a/**/b").unwrap(), keyexpr::new("c").unwrap()))
    );
    assert_eq!(
        splits.next(),
        Some((
            keyexpr::new("a/**").unwrap(),
            keyexpr::new("**/b/c").unwrap()
        ))
    );
    assert_eq!(splits.next(), None);
    let ke = keyexpr::new("**").unwrap();
    let mut splits = ke.iter_splits_ltr();
    assert_eq!(
        splits.next(),
        Some((keyexpr::new("**").unwrap(), keyexpr::new("**").unwrap()))
    );
    assert_eq!(splits.next(), None);
    let ke = keyexpr::new("ab").unwrap();
    let mut splits = ke.iter_splits_ltr();
    assert_eq!(splits.next(), None);
    let ke = keyexpr::new("ab/cd").unwrap();
    let mut splits = ke.iter_splits_ltr();
    assert_eq!(
        splits.next(),
        Some((keyexpr::new("ab").unwrap(), keyexpr::new("cd").unwrap()))
    );
    assert_eq!(splits.next(), None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chunks<'a> {
    inner: &'a str,
}

impl<'a> Chunks<'a> {
    /// Convert the remaining part of the iterator to a keyexpr if it is not empty.
    pub const fn as_keyexpr(self) -> Option<&'a keyexpr> {
        match self.inner.is_empty() {
            true => None,
            _ => Some(unsafe { keyexpr::from_str_unchecked(self.inner) }),
        }
    }
    /// Peek at the next chunk without consuming it.
    pub fn peek(&self) -> Option<&keyexpr> {
        if self.inner.is_empty() {
            None
        } else {
            Some(unsafe {
                keyexpr::from_str_unchecked(
                    &self.inner[..self.inner.find('/').unwrap_or(self.inner.len())],
                )
            })
        }
    }
    /// Peek at the last chunk without consuming it.
    pub fn peek_back(&self) -> Option<&keyexpr> {
        if self.inner.is_empty() {
            None
        } else {
            Some(unsafe {
                keyexpr::from_str_unchecked(
                    &self.inner[self.inner.rfind('/').map_or(0, |i| i + 1)..],
                )
            })
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a keyexpr;
    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }
        let (next, inner) = self.inner.split_once('/').unwrap_or((self.inner, ""));
        self.inner = inner;
        Some(unsafe { keyexpr::from_str_unchecked(next) })
    }
}

impl DoubleEndedIterator for Chunks<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }
        let (inner, next) = self.inner.rsplit_once('/').unwrap_or(("", self.inner));
        self.inner = inner;
        Some(unsafe { keyexpr::from_str_unchecked(next) })
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
            Self::LoneDollarStar => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: empty chunks are forbidden, as well as leading and trailing slashes"),
            Self::SingleStarAfterDoubleStar => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `**/*` must be replaced by `*/**` to reach canon-form"),
            Self::DoubleStarAfterDoubleStar => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `**/**` must be replaced by `**` to reach canon-form"),
            Self::EmptyChunk => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: empty chunks are forbidden, as well as leading and trailing slashes"),
            Self::StarInChunk => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `*` may only be preceded by `/` or `$`"),
            Self::DollarAfterDollar => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `$` is not allowed after `$*`"),
            Self::SharpOrQMark => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `#` and `?` are forbidden characters"),
            Self::UnboundDollar => zerr!(ZE::KeyExprValidation).context("Invalid Key Expr: `$` is only allowed in `$*`")
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
#[test]
fn autocanon() {
    use core::str::FromStr;

    let mut s: String<16> = String::from_str("hello/**/*").unwrap();
    let mut s: &mut str = &mut s;
    assert_eq!(keyexpr::autocanonize(&mut s).unwrap(), "hello/*/**");
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
    /// Attempts to construct a non-wild key expression from anything convertible to keyexpression.
    ///
    /// Will return an Err if `t` isn't a valid key expression.
    pub fn new<'a, T, E>(t: &'a T) -> Result<&'a Self, ZError>
    where
        &'a keyexpr: TryFrom<&'a T, Error = E>,
        E: Into<ZError>,
        T: ?Sized,
    {
        let ke: &'a keyexpr = t.try_into().map_err(|e: E| e.into())?;
        ke.try_into()
    }

    /// # Safety
    /// This constructs a [`nonwild_keyexpr`] without ensuring that it is a valid key-expression without wild chunks.
    ///
    /// Much like [`core::str::from_utf8_unchecked`], this is memory-safe, but calling this without maintaining
    /// [`nonwild_keyexpr`]'s invariants yourself may lead to unexpected behaviors, the Zenoh network dropping your messages.
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
            bail!(ZE::NonWildExprContainsWildChunks);
        }
        Ok(unsafe { core::mem::transmute::<&keyexpr, &nonwild_keyexpr>(value) })
    }
}

impl<const N: usize> Borrow<nonwild_keyexpr> for OwnedNonWildKeyExpr<N> {
    fn borrow(&self) -> &nonwild_keyexpr {
        self
    }
}
