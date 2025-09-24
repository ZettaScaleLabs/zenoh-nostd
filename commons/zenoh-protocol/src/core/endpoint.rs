use core::{borrow::Borrow, convert::TryFrom, fmt, str::FromStr};

use heapless::{format, String};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

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

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct ProtocolMut<'a, const N: usize>(&'a mut EndPoint<N>);

impl<'a, const N: usize> ProtocolMut<'a, N> {
    pub fn as_str(&'a self) -> &'a str {
        address(self.0.as_str())
    }

    pub fn set(&mut self, p: &str) -> ZResult<()> {
        let ep = EndPoint::new(p, self.0.address(), self.0.metadata(), self.0.config())?;

        self.0.inner = ep.inner;
        Ok(())
    }
}

impl<const N: usize> AsRef<str> for ProtocolMut<'_, N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for ProtocolMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for ProtocolMut<'_, N> {
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

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct AddressMut<'a, const N: usize>(&'a mut EndPoint<N>);

impl<'a, const N: usize> AddressMut<'a, N> {
    pub fn as_str(&'a self) -> &'a str {
        address(self.0.as_str())
    }

    pub fn set(&'a mut self, a: &str) -> ZResult<()> {
        let ep = EndPoint::new(self.0.protocol(), a, self.0.metadata(), self.0.config())?;

        self.0.inner = ep.inner;
        Ok(())
    }
}

impl<const N: usize> AsRef<str> for AddressMut<'_, N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for AddressMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for AddressMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

// Metadata
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Metadata<'a>(pub(super) &'a str);

impl<'a> Metadata<'a> {
    pub const RELIABILITY: &'static str = "rel";
    pub const PRIORITIES: &'static str = "prio";

    pub fn as_str(&self) -> &'a str {
        self.0
    }

    pub fn is_empty(&'a self) -> bool {
        self.as_str().is_empty()
    }

    pub fn iter(&'a self) -> impl DoubleEndedIterator<Item = (&'a str, &'a str)> + Clone {
        parameters::iter(self.0)
    }

    pub fn get(&'a self, k: &str) -> Option<&'a str> {
        parameters::get(self.0, k)
    }

    pub fn values(&'a self, k: &str) -> impl DoubleEndedIterator<Item = &'a str> {
        parameters::values(self.0, k)
    }
}

impl AsRef<str> for Metadata<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Metadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Metadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct MetadataMut<'a, const N: usize>(&'a mut EndPoint<N>);

impl<'a, const N: usize> MetadataMut<'a, N> {
    pub fn as_str(&'a self) -> &'a str {
        metadata(self.0.as_str())
    }

    pub fn is_empty(&'a self) -> bool {
        self.as_str().is_empty()
    }
}

impl<const N: usize> MetadataMut<'_, N> {
    pub fn extend_from_iter<'s, I, K, V>(&mut self, iter: I) -> ZResult<()>
    where
        I: Iterator<Item = (&'s K, &'s V)> + Clone,
        K: Borrow<str> + 's + ?Sized,
        V: Borrow<str> + 's + ?Sized,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            parameters::from_iter::<_, N>(parameters::sort::<_, N>(parameters::join(
                self.0.metadata().iter(),
                iter.map(|(k, v)| (k.borrow(), v.borrow())),
            )))?,
            self.0.config(),
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }

    pub fn insert<K, V>(&mut self, k: K, v: V) -> ZResult<()>
    where
        K: Borrow<str>,
        V: Borrow<str>,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            parameters::insert_sort::<N>(self.0.metadata().as_str(), k.borrow(), v.borrow()).0?,
            self.0.config(),
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }

    pub fn remove<K>(&mut self, k: K) -> ZResult<()>
    where
        K: Borrow<str>,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            parameters::remove::<N>(self.0.metadata().as_str(), k.borrow()).0?,
            self.0.config(),
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }
}

impl<const N: usize> AsRef<str> for MetadataMut<'_, N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for MetadataMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for MetadataMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

// Config
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Config<'a>(pub(super) &'a str);

impl<'a> Config<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    pub fn is_empty(&'a self) -> bool {
        self.as_str().is_empty()
    }

    pub fn iter(&'a self) -> impl DoubleEndedIterator<Item = (&'a str, &'a str)> + Clone {
        parameters::iter(self.0)
    }

    pub fn get(&'a self, k: &str) -> Option<&'a str> {
        parameters::get(self.0, k)
    }

    pub fn values(&'a self, k: &str) -> impl DoubleEndedIterator<Item = &'a str> {
        parameters::values(self.0, k)
    }
}

impl AsRef<str> for Config<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Config<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct ConfigMut<'a, const N: usize>(&'a mut EndPoint<N>);

impl<'a, const N: usize> ConfigMut<'a, N> {
    pub fn as_str(&'a self) -> &'a str {
        config(self.0.as_str())
    }

    pub fn is_empty(&'a self) -> bool {
        self.as_str().is_empty()
    }
}

impl<const N: usize> ConfigMut<'_, N> {
    pub fn extend_from_iter<'s, I, K, V>(&mut self, iter: I) -> ZResult<()>
    where
        I: Iterator<Item = (&'s K, &'s V)> + Clone,
        K: Borrow<str> + 's + ?Sized,
        V: Borrow<str> + 's + ?Sized,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            self.0.metadata(),
            parameters::from_iter::<'_, _, N>(parameters::sort::<'_, _, N>(parameters::join(
                self.0.config().iter(),
                iter.map(|(k, v)| (k.borrow(), v.borrow())),
            )))?,
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }

    pub fn insert<'s, K, V>(&mut self, k: K, v: V) -> ZResult<()>
    where
        K: Borrow<str>,
        V: Borrow<str>,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            self.0.metadata(),
            parameters::insert_sort::<N>(self.0.config().as_str(), k.borrow(), v.borrow()).0?,
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }

    pub fn remove<K>(&mut self, k: K) -> ZResult<()>
    where
        K: Borrow<str>,
    {
        let ep = EndPoint::new(
            self.0.protocol(),
            self.0.address(),
            self.0.metadata(),
            parameters::remove::<N>(self.0.config().as_str(), k.borrow()).0?,
        )?;

        self.0.inner = ep.inner;
        Ok(())
    }
}

impl<const N: usize> AsRef<str> for ConfigMut<'_, N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for ConfigMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for ConfigMut<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A string that respects the [`EndPoint`] canon form: `<locator>[#<config>]`.
///
/// `<locator>` is a valid [`Locator`] and `<config>` is of the form
/// `<key1>=<value1>;...;<keyN>=<valueN>` where keys are alphabetically sorted. `<config>` is
/// optional and can be provided to configure some aspects for an [`EndPoint`], e.g. the interface
/// to listen on or connect to.
///
/// A full [`EndPoint`] string is hence in the form of `<proto>/<address>[?<metadata>][#config]`.
///
/// ## Metadata
///
/// - **`prio`**: a priority range bounded inclusively below and above (e.g. `2-4` signifies
///   priorities 2, 3 and 4). This value is used to select the link used for transmission based on
///   the Priority of the message in question.
///
///   For example, `tcp/localhost:7447?prio=1-3` assigns priorities
///   [`crate::core::Priority::RealTime`], [`crate::core::Priority::InteractiveHigh`] and
///   [`crate::core::Priority::InteractiveLow`] to the established link.
///
/// - **`rel`**: either "0" for [`crate::core::Reliability::BestEffort`] or "1" for
///   [`crate::core::Reliability::Reliable`]. This value is used to select the link used for
///   transmission based on the reliability of the message in question.
///
///   For example, `tcp/localhost:7447?prio=6-7;rel=0` assigns priorities
///   [`crate::core::Priority::DataLow`] and [`crate::core::Priority::Background`], and
///   [`crate::core::Reliability::BestEffort`] to the established link.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EndPoint<const N: usize> {
    pub(super) inner: String<N>,
}

impl<const N: usize> EndPoint<N> {
    pub fn new<A, B, C, D>(protocol: A, address: B, metadata: C, config: D) -> ZResult<Self>
    where
        A: AsRef<str>,
        B: AsRef<str>,
        C: AsRef<str>,
        D: AsRef<str>,
    {
        let p: &str = protocol.as_ref();
        let a: &str = address.as_ref();
        let m: &str = metadata.as_ref();
        let c: &str = config.as_ref();

        let len = p.len() + a.len() + m.len();
        if len > u8::MAX as usize {
            bail!(ZE::EndPointTooBig);
        }

        let s: String<N> = match (m.is_empty(), c.is_empty()) {
            (true, true) => format!("{p}{PROTO_SEPARATOR}{a}"),
            (false, true) => format!("{p}{PROTO_SEPARATOR}{a}{METADATA_SEPARATOR}{m}"),
            (true, false) => format!("{p}{PROTO_SEPARATOR}{a}{CONFIG_SEPARATOR}{c}"),
            (false, false) => {
                format!("{p}{PROTO_SEPARATOR}{a}{METADATA_SEPARATOR}{m}{CONFIG_SEPARATOR}{c}")
            }
        }
        .map_err(|_| zerr!(ZE::CapacityExceeded))?;

        Self::try_from(s)
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn split(&self) -> (Protocol<'_>, Address<'_>, Metadata<'_>, Config<'_>) {
        (
            self.protocol(),
            self.address(),
            self.metadata(),
            self.config(),
        )
    }

    pub fn protocol(&self) -> Protocol<'_> {
        Protocol(protocol(self.inner.as_str()))
    }

    pub fn protocol_mut(&mut self) -> ProtocolMut<'_, N> {
        ProtocolMut(self)
    }

    pub fn address(&self) -> Address<'_> {
        Address(address(self.inner.as_str()))
    }

    pub fn address_mut(&mut self) -> AddressMut<'_, N> {
        AddressMut(self)
    }

    pub fn metadata(&self) -> Metadata<'_> {
        Metadata(metadata(self.inner.as_str()))
    }

    pub fn metadata_mut(&mut self) -> MetadataMut<'_, N> {
        MetadataMut(self)
    }

    pub fn config(&self) -> Config<'_> {
        Config(config(self.inner.as_str()))
    }

    pub fn config_mut(&mut self) -> ConfigMut<'_, N> {
        ConfigMut(self)
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
