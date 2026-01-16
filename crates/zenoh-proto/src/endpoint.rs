use core::{convert::TryFrom, fmt};

const PROTO_SEPARATOR: char = '/';
const METADATA_SEPARATOR: char = '?';
const CONFIG_SEPARATOR: char = '#';

fn protocol(s: &str) -> &str {
    let pdix = s.find(PROTO_SEPARATOR).unwrap_or(s.len());
    &s[..pdix]
}

fn address(s: &str) -> &str {
    let pdix = s.find(PROTO_SEPARATOR).unwrap_or(s.len());
    let midx = s.find(METADATA_SEPARATOR).unwrap_or(s.len());
    let cidx = s.find(CONFIG_SEPARATOR).unwrap_or(s.len());
    &s[pdix + 1..midx.min(cidx)]
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Protocol<'a>(&'a str);

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
pub struct Address<'a>(&'a str);

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
pub struct EndPoint<'a> {
    pub(super) inner: &'a str,
}

impl EndPoint<'_> {
    pub fn protocol(&self) -> Protocol<'_> {
        Protocol(protocol(self.inner))
    }

    pub fn address(&self) -> Address<'_> {
        Address(address(self.inner))
    }
}

impl fmt::Display for EndPoint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.inner)
    }
}

impl fmt::Debug for EndPoint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl<'a> TryFrom<&'a str> for EndPoint<'a> {
    type Error = crate::EndpointError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let pidx = s
            .find(PROTO_SEPARATOR)
            .and_then(|i| (!s[..i].is_empty() && !s[i + 1..].is_empty()).then_some(i))
            .ok_or(crate::EndpointError::NoProtocolSeparator)?;

        match (s.find(METADATA_SEPARATOR), s.find(CONFIG_SEPARATOR)) {
            (None, None) => Ok(EndPoint { inner: s }),

            (Some(midx), None) if midx > pidx && !s[midx + 1..].is_empty() => {
                crate::zbail!(crate::EndpointError::MetadataNotSupported)
            }

            (None, Some(cidx)) if cidx > pidx && !s[cidx + 1..].is_empty() => {
                crate::zbail!(crate::EndpointError::ConfigNotSupported)
            }

            (Some(midx), Some(cidx))
                if midx > pidx
                    && cidx > midx
                    && !s[midx + 1..cidx].is_empty()
                    && !s[cidx + 1..].is_empty() =>
            {
                crate::zbail!(crate::EndpointError::MetadataNotSupported)
            }
            _ => Err(crate::EndpointError::MetadataNotSupported),
        }
    }
}
