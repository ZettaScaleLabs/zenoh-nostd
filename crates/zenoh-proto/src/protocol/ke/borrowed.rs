use core::{convert::TryFrom, fmt, ops::Deref};

use crate::{ZResult, protocol::ke::ZKeyExprError, zbail};

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct keyexpr(str);

impl keyexpr {
    pub fn new(v: &str) -> ZResult<&'_ Self, ZKeyExprError> {
        if v.is_empty() || v.ends_with('/') {
            zbail!(ZKeyExprError::EmptyChunk);
        }

        let bytes = v.as_bytes();

        let mut chunk_start = 0;

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                c if c > b'/' && c != b'?' => i += 1,

                b'/' if i == chunk_start => zbail!(ZKeyExprError::EmptyChunk),

                b'/' => {
                    i += 1;
                    chunk_start = i;
                }

                b'*' if i != chunk_start => zbail!(ZKeyExprError::StarInChunk),

                b'*' => match bytes.get(i + 1) {
                    None => break,

                    Some(&b'/') => {
                        i += 2;
                        chunk_start = i;
                    }

                    Some(&b'*') => match bytes.get(i + 2) {
                        None => break,

                        Some(&b'/') if matches!(bytes.get(i + 3), Some(b'*')) => {
                            #[cold]
                            fn double_star_err(v: &str, i: usize) -> ZKeyExprError {
                                match (v.as_bytes().get(i + 4), v.as_bytes().get(i + 5)) {
                                    (None | Some(&b'/'), _) => {
                                        ZKeyExprError::SingleStarAfterDoubleStar
                                    }
                                    (Some(&b'*'), None | Some(&b'/')) => {
                                        ZKeyExprError::DoubleStarAfterDoubleStar
                                    }
                                    _ => ZKeyExprError::StarInChunk,
                                }
                            }

                            zbail!(double_star_err(v, i));
                        }

                        Some(&b'/') => {
                            i += 3;
                            chunk_start = i;
                        }

                        _ => zbail!(ZKeyExprError::StarInChunk),
                    },

                    _ => zbail!(ZKeyExprError::StarInChunk),
                },

                b'$' if bytes.get(i + 1) != Some(&b'*') => {
                    zbail!(ZKeyExprError::UnboundDollar)
                }

                b'$' => match bytes.get(i + 2) {
                    Some(&b'$') => zbail!(ZKeyExprError::DollarAfterDollar),

                    Some(&b'/') | None if i == chunk_start => {
                        zbail!(ZKeyExprError::LoneDollarStar)
                    }

                    None => break,

                    _ => i += 2,
                },

                b'#' | b'?' => zbail!(ZKeyExprError::SharpOrQMark),

                _ => i += 1,
            }
        }

        Ok(keyexpr::from_str_unchecked(v))
    }

    pub(crate) fn is_wild_impl(&self) -> bool {
        self.0.contains(super::SINGLE_WILD as char)
    }

    pub const fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn from_str_unchecked(s: &str) -> &Self {
        unsafe { core::mem::transmute(s) }
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
pub struct nonwild_keyexpr(keyexpr);

impl nonwild_keyexpr {}

impl Deref for nonwild_keyexpr {
    type Target = keyexpr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<&'a keyexpr> for &'a nonwild_keyexpr {
    type Error = ZKeyExprError;
    fn try_from(v: &'a keyexpr) -> Result<Self, Self::Error> {
        if v.is_wild_impl() {
            zbail!(ZKeyExprError::WildChunk);
        }

        Ok(unsafe { core::mem::transmute::<&keyexpr, &nonwild_keyexpr>(v) })
    }
}
