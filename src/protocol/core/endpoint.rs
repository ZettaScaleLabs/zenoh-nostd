use core::{convert::TryFrom, fmt};

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
pub struct EndPoint {
    pub(super) inner: &'static str,
}

impl EndPoint {
    pub fn new(
        protocol: &'static str,
        address: &'static str,
    ) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
        let len = protocol.len() + address.len();
        if len > u8::MAX as usize {
            crate::zbail!(crate::protocol::ZProtocolError::Invalid);
        }

        let s: &'static str = format_args!("{protocol}{PROTO_SEPARATOR}{address}")
            .as_str()
            .ok_or(crate::protocol::ZProtocolError::Invalid)?;

        Self::try_from(s)
    }

    pub fn as_str(&self) -> &str {
        self.inner
    }

    pub fn split(&self) -> (Protocol<'_>, Address<'_>) {
        (self.protocol(), self.address())
    }

    pub fn protocol(&self) -> Protocol<'_> {
        Protocol(protocol(self.inner))
    }

    pub fn address(&self) -> Address<'_> {
        Address(address(self.inner))
    }
}

impl fmt::Display for EndPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.inner)
    }
}

impl fmt::Debug for EndPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl TryFrom<&'static str> for EndPoint {
    type Error = crate::protocol::ZProtocolError;

    fn try_from(s: &'static str) -> Result<Self, Self::Error> {
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
