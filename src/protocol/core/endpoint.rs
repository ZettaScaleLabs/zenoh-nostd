use core::{convert::TryFrom, fmt, str::FromStr};

use heapless::{String, format};

use super::locator::*;

pub const PROTO_SEPARATOR: char = '/';
pub const METADATA_SEPARATOR: char = '?';
pub const CONFIG_SEPARATOR: char = '#';

pub(super) fn protocol(s: &str) -> &str {
    let pdix = s.find(PROTO_SEPARATOR).unwrap_or(s.len());
    &s[..pdix]
}

pub(super) fn address(s: &str) -> &str {
    let pdix = s.find(PROTO_SEPARATOR).unwrap_or(s.len());
    let midx = s.find(METADATA_SEPARATOR).unwrap_or(s.len());
    let cidx = s.find(CONFIG_SEPARATOR).unwrap_or(s.len());
    &s[pdix + 1..midx.min(cidx)]
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Protocol<'a>(pub(super) &'a str);

impl<'a> Protocol<'a> {
    pub fn as_str(&self) -> &'_ str {
        self.0
    }
}

impl AsRef<str> for Protocol<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Protocol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Protocol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address<'a>(pub(super) &'a str);

impl<'a> Address<'a> {
    pub fn as_str(&self) -> &'_ str {
        self.0
    }
}

impl AsRef<str> for Address<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Address<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Address<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl<'a> From<&'a str> for Address<'a> {
    fn from(value: &'a str) -> Self {
        Address(value)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EndPoint<const N: usize> {
    pub(super) inner: String<N>,
}

impl<const N: usize> EndPoint<N> {
    pub fn new<A, B>(
        protocol: A,
        address: B,
    ) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError>
    where
        A: AsRef<str>,
        B: AsRef<str>,
    {
        let p: &str = protocol.as_ref();
        let a: &str = address.as_ref();

        let len = p.len() + a.len();
        if len > u8::MAX as usize {
            crate::zbail!(crate::protocol::ZProtocolError::Invalid);
        }

        let s: String<N> = format!("{p}{PROTO_SEPARATOR}{a}")
            .map_err(|_| crate::protocol::ZProtocolError::Invalid)?;

        Self::try_from(s)
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn split(&self) -> (Protocol<'_>, Address<'_>) {
        (self.protocol(), self.address())
    }

    pub fn protocol(&self) -> Protocol<'_> {
        Protocol(protocol(self.inner.as_str()))
    }

    pub fn address(&self) -> Address<'_> {
        Address(address(self.inner.as_str()))
    }

    pub fn to_locator(&self) -> Locator<N> {
        self.clone().into()
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::{
            Rng,
            distributions::{Alphanumeric, DistString},
        };

        const MIN: usize = 1;
        const MAX: usize = 8;

        let mut rng = rand::thread_rng();
        let mut endpoint = String::<{ MAX * 2 + 1 }>::new();

        let len = rng.gen_range(MIN..MAX);
        let proto = Alphanumeric.sample_string(&mut rng, len);
        endpoint.push_str(proto.as_str()).unwrap();

        endpoint.push(PROTO_SEPARATOR).unwrap();

        let len = rng.gen_range(MIN..MAX);
        let address = Alphanumeric.sample_string(&mut rng, len);
        endpoint.push_str(address.as_str()).unwrap();

        endpoint.parse().unwrap()
    }
}

impl<const N: usize> From<Locator<N>> for EndPoint<N> {
    fn from(val: Locator<N>) -> Self {
        val.0
    }
}

impl<const N: usize> fmt::Display for EndPoint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

impl<const N: usize> fmt::Debug for EndPoint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl<const N: usize> From<EndPoint<N>> for String<N> {
    fn from(v: EndPoint<N>) -> String<N> {
        v.inner
    }
}

impl<const N: usize> TryFrom<String<N>> for EndPoint<N> {
    type Error = crate::protocol::ZProtocolError;

    fn try_from(s: String<N>) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
        let pidx = s
            .find(PROTO_SEPARATOR)
            .and_then(|i| (!s[..i].is_empty() && !s[i + 1..].is_empty()).then_some(i))
            .ok_or(crate::protocol::ZProtocolError::Invalid)?;

        match (s.find(METADATA_SEPARATOR), s.find(CONFIG_SEPARATOR)) {
            (None, None) => Ok(EndPoint { inner: s }),

            (Some(midx), None) if midx > pidx && !s[midx + 1..].is_empty() => {
                crate::zbail!(crate::protocol::ZProtocolError::Invalid)
            }

            (None, Some(cidx)) if cidx > pidx && !s[cidx + 1..].is_empty() => {
                crate::zbail!(crate::protocol::ZProtocolError::Invalid)
            }

            (Some(midx), Some(cidx))
                if midx > pidx
                    && cidx > midx
                    && !s[midx + 1..cidx].is_empty()
                    && !s[cidx + 1..].is_empty() =>
            {
                crate::zbail!(crate::protocol::ZProtocolError::Invalid)
            }
            _ => Err(crate::protocol::ZProtocolError::Invalid),
        }
    }
}

impl<const N: usize> FromStr for EndPoint<N> {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from_str(s).map_err(|_| crate::protocol::ZProtocolError::Invalid)?)
    }
}
