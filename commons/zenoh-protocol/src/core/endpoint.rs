use core::{borrow::Borrow, convert::TryFrom, fmt, str::FromStr};

use heapless::{format, String};
use zenoh_result::{zbail, zerr, ZError, ZResult, ZE};

use super::{locator::*, parameters};

// Parsing chars
pub const PROTO_SEPARATOR: char = '/';
pub const METADATA_SEPARATOR: char = '?';
pub const CONFIG_SEPARATOR: char = '#';

// Parsing functions
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

pub(super) fn metadata(s: &str) -> &str {
    match s.find(METADATA_SEPARATOR) {
        Some(midx) => {
            let cidx = s.find(CONFIG_SEPARATOR).unwrap_or(s.len());
            &s[midx + 1..cidx]
        }
        None => "",
    }
}

pub(super) fn config(s: &str) -> &str {
    match s.find(CONFIG_SEPARATOR) {
        Some(cidx) => &s[cidx + 1..],
        None => "",
    }
}

// Protocol
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Protocol<'a>(pub(super) &'a str);

impl<'a> Protocol<'a> {
    pub fn as_str(&self) -> &'a str {
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

// Address
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address<'a>(pub(super) &'a str);

impl<'a> Address<'a> {
    pub fn as_str(&self) -> &'a str {
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
    pub fn new<A, B>(protocol: A, address: B) -> ZResult<Self>
    where
        A: AsRef<str>,
        B: AsRef<str>,
    {
        let p: &str = protocol.as_ref();
        let a: &str = address.as_ref();

        let len = p.len() + a.len();
        if len > u8::MAX as usize {
            zbail!(ZE::EndPointTooBig);
        }

        let s: String<N> =
            format!("{p}{PROTO_SEPARATOR}{a}").map_err(|_| zerr!(ZE::CapacityExceeded))?;

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
    type Error = ZError;

    fn try_from(s: String<N>) -> Result<Self, Self::Error> {
        let pidx = s
            .find(PROTO_SEPARATOR)
            .and_then(|i| (!s[..i].is_empty() && !s[i + 1..].is_empty()).then_some(i))
            .ok_or_else(|| zerr!(ZE::InvalidEndPoint))?;

        match (s.find(METADATA_SEPARATOR), s.find(CONFIG_SEPARATOR)) {
            // No metadata or config at all
            (None, None) => Ok(EndPoint { inner: s }),
            // There is some metadata
            (Some(midx), None) if midx > pidx && !s[midx + 1..].is_empty() => {
                let mut inner = String::<N>::new();
                inner
                    .push_str(&s[..midx + 1])
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?; // Includes metadata separator
                parameters::from_iter_into(
                    parameters::sort::<_, N>(parameters::iter(&s[midx + 1..])),
                    &mut inner,
                )?;
                Ok(EndPoint { inner })
            }
            // There is some config
            (None, Some(cidx)) if cidx > pidx && !s[cidx + 1..].is_empty() => {
                let mut inner = String::<N>::new();
                inner
                    .push_str(&s[..cidx + 1])
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?; // Includes config separator
                parameters::from_iter_into(
                    parameters::sort::<_, N>(parameters::iter(&s[cidx + 1..])),
                    &mut inner,
                )?;
                Ok(EndPoint { inner })
            }
            // There is some metadata and some config
            (Some(midx), Some(cidx))
                if midx > pidx
                    && cidx > midx
                    && !s[midx + 1..cidx].is_empty()
                    && !s[cidx + 1..].is_empty() =>
            {
                let mut inner = String::<N>::new();
                inner
                    .push_str(&s[..midx + 1])
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?; // Includes metadata separator

                parameters::from_iter_into(
                    parameters::sort::<_, N>(parameters::iter(&s[midx + 1..cidx])),
                    &mut inner,
                )?;

                inner
                    .push(CONFIG_SEPARATOR)
                    .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                parameters::from_iter_into(
                    parameters::sort::<_, N>(parameters::iter(&s[cidx + 1..])),
                    &mut inner,
                )?;

                Ok(EndPoint { inner })
            }
            _ => Err(zerr!(ZE::InvalidEndPoint)),
        }
    }
}

impl<const N: usize> FromStr for EndPoint<N> {
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))?)
    }
}

#[test]
fn endpoints() {
    type EndPoint = super::EndPoint<256>;

    assert!(EndPoint::from_str("/").is_err());
    assert!(EndPoint::from_str("?").is_err());
    assert!(EndPoint::from_str("#").is_err());

    assert!(EndPoint::from_str("udp").is_err());
    assert!(EndPoint::from_str("/udp").is_err());
    assert!(EndPoint::from_str("udp/").is_err());

    assert!(EndPoint::from_str("udp/127.0.0.1:7447?").is_err());
    assert!(EndPoint::from_str("udp?127.0.0.1:7447").is_err());
    assert!(EndPoint::from_str("udp?127.0.0.1:7447/meta").is_err());

    assert!(EndPoint::from_str("udp/127.0.0.1:7447#").is_err());
    assert!(EndPoint::from_str("udp/127.0.0.1:7447?#").is_err());
    assert!(EndPoint::from_str("udp/127.0.0.1:7447#?").is_err());
    assert!(EndPoint::from_str("udp#127.0.0.1:7447/").is_err());
    assert!(EndPoint::from_str("udp#127.0.0.1:7447/?").is_err());
    assert!(EndPoint::from_str("udp/127.0.0.1:7447?a=1#").is_err());

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert!(endpoint.metadata().as_str().is_empty());
    assert_eq!(endpoint.metadata().iter().count(), 0);

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447?a=1;b=2").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert_eq!(endpoint.metadata().as_str(), "a=1;b=2");
    assert_eq!(endpoint.metadata().iter().count(), 2);
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("a", "1"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("a"), Some("1"));
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("b", "2"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("b"), Some("2"));
    assert!(endpoint.config().as_str().is_empty());
    assert_eq!(endpoint.config().iter().count(), 0);

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447?b=2;a=1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert_eq!(endpoint.metadata().as_str(), "a=1;b=2");
    assert_eq!(endpoint.metadata().iter().count(), 2);
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("a", "1"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("a"), Some("1"));
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("b", "2"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("a"), Some("1"));
    assert!(endpoint.config().as_str().is_empty());
    assert_eq!(endpoint.config().iter().count(), 0);

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447#A=1;B=2").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447#A=1;B=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert!(endpoint.metadata().as_str().is_empty());
    assert_eq!(endpoint.metadata().iter().count(), 0);
    assert_eq!(endpoint.config().as_str(), "A=1;B=2");
    assert_eq!(endpoint.config().iter().count(), 2);
    endpoint.config().iter().find(|x| x == &("A", "1")).unwrap();
    assert_eq!(endpoint.config().get("A"), Some("1"));
    endpoint.config().iter().find(|x| x == &("B", "2")).unwrap();
    assert_eq!(endpoint.config().get("B"), Some("2"));

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447#B=2;A=1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447#A=1;B=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert!(endpoint.metadata().as_str().is_empty());
    assert_eq!(endpoint.metadata().iter().count(), 0);
    assert_eq!(endpoint.config().as_str(), "A=1;B=2");
    assert_eq!(endpoint.config().iter().count(), 2);
    endpoint.config().iter().find(|x| x == &("A", "1")).unwrap();
    assert_eq!(endpoint.config().get("A"), Some("1"));
    endpoint.config().iter().find(|x| x == &("B", "2")).unwrap();
    assert_eq!(endpoint.config().get("B"), Some("2"));

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447?a=1;b=2#A=1;B=2").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2#A=1;B=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert_eq!(endpoint.metadata().as_str(), "a=1;b=2");
    assert_eq!(endpoint.metadata().iter().count(), 2);
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("a", "1"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("a"), Some("1"));
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("b", "2"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("b"), Some("2"));
    assert_eq!(endpoint.config().as_str(), "A=1;B=2");
    assert_eq!(endpoint.config().iter().count(), 2);
    endpoint.config().iter().find(|x| x == &("A", "1")).unwrap();
    assert_eq!(endpoint.config().get("A"), Some("1"));
    endpoint.config().iter().find(|x| x == &("B", "2")).unwrap();
    assert_eq!(endpoint.config().get("B"), Some("2"));

    let endpoint = EndPoint::from_str("udp/127.0.0.1:7447?b=2;a=1#B=2;A=1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2#A=1;B=2");
    assert_eq!(endpoint.protocol().as_str(), "udp");
    assert_eq!(endpoint.address().as_str(), "127.0.0.1:7447");
    assert_eq!(endpoint.metadata().as_str(), "a=1;b=2");
    assert_eq!(endpoint.metadata().iter().count(), 2);
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("a", "1"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("a"), Some("1"));
    endpoint
        .metadata()
        .iter()
        .find(|x| x == &("b", "2"))
        .unwrap();
    assert_eq!(endpoint.metadata().get("b"), Some("2"));
    assert_eq!(endpoint.config().as_str(), "A=1;B=2");
    assert_eq!(endpoint.config().iter().count(), 2);
    endpoint.config().iter().find(|x| x == &("A", "1")).unwrap();
    assert_eq!(endpoint.config().get("A"), Some("1"));
    endpoint.config().iter().find(|x| x == &("B", "2")).unwrap();
    assert_eq!(endpoint.config().get("B"), Some("2"));

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447?a=1;b=2").unwrap();
    endpoint.metadata_mut().insert("c", "3").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2;c=3");

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447?b=2;c=3").unwrap();
    endpoint.metadata_mut().insert("a", "1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2;c=3");

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447?a=1;b=2").unwrap();
    endpoint.config_mut().insert("A", "1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2#A=1");

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447?b=2;c=3#B=2").unwrap();
    endpoint.config_mut().insert("A", "1").unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?b=2;c=3#A=1;B=2");

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447").unwrap();
    endpoint
        .metadata_mut()
        .extend_from_iter([("a", "1"), ("c", "3"), ("b", "2")].iter().copied())
        .unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447?a=1;b=2;c=3");

    let mut endpoint = EndPoint::from_str("udp/127.0.0.1:7447").unwrap();
    endpoint
        .config_mut()
        .extend_from_iter([("A", "1"), ("C", "3"), ("B", "2")].iter().copied())
        .unwrap();
    assert_eq!(endpoint.as_str(), "udp/127.0.0.1:7447#A=1;B=2;C=3");

    let endpoint =
        EndPoint::from_str("udp/127.0.0.1:7447#iface=en0;join=224.0.0.1|224.0.0.2|224.0.0.3")
            .unwrap();
    let c = endpoint.config();
    assert_eq!(c.get("iface"), Some("en0"));
    assert_eq!(c.get("join"), Some("224.0.0.1|224.0.0.2|224.0.0.3"));
    assert_eq!(c.values("iface").count(), 1);
    let mut i = c.values("iface");
    assert_eq!(i.next(), Some("en0"));
    assert_eq!(c.values("join").count(), 3);
    let mut i = c.values("join");
    assert_eq!(i.next(), Some("224.0.0.1"));
    assert_eq!(i.next(), Some("224.0.0.2"));
    assert_eq!(i.next(), Some("224.0.0.3"));
    assert_eq!(i.next(), None);
}
