use core::{convert::TryFrom, fmt, hash::Hash, str::FromStr};

use heapless::String;
use zenoh_result::{zerr, ZError, ZResult, ZE};

use super::endpoint::*;

/// A string that respects the [`Locator`] canon form: `<proto>/<address>[?<metadata>]`.
///
/// `<metadata>` is of the form `<key1>=<value1>;...;<keyN>=<valueN>` where keys are alphabetically sorted.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Locator<const N: usize>(pub(super) EndPoint<N>);

impl<const N: usize> Locator<N> {
    pub fn new<A, B, C>(protocol: A, address: B, metadata: C) -> ZResult<Self>
    where
        A: AsRef<str>,
        B: AsRef<str>,
        C: AsRef<str>,
    {
        let ep = EndPoint::new(protocol, address, metadata, "")?;
        Ok(Self(ep))
    }

    pub fn protocol(&self) -> Protocol<'_> {
        self.0.protocol()
    }

    pub fn protocol_mut(&mut self) -> ProtocolMut<'_, N> {
        self.0.protocol_mut()
    }

    pub fn address(&self) -> Address<'_> {
        self.0.address()
    }

    pub fn address_mut(&mut self) -> AddressMut<'_, N> {
        self.0.address_mut()
    }

    pub fn metadata(&self) -> Metadata<'_> {
        self.0.metadata()
    }

    pub fn metadata_mut(&mut self) -> MetadataMut<'_, N> {
        self.0.metadata_mut()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_endpoint(&self) -> EndPoint<N> {
        self.0.clone()
    }
}

impl<const N: usize> From<EndPoint<N>> for Locator<N> {
    fn from(mut val: EndPoint<N>) -> Self {
        if let Some(cidx) = val.inner.find(CONFIG_SEPARATOR) {
            val.inner.truncate(cidx);
        }
        Locator(val)
    }
}

impl<const N: usize> TryFrom<&str> for Locator<N> {
    type Error = ZError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))?)
    }
}

impl<const N: usize> From<Locator<N>> for String<N> {
    fn from(val: Locator<N>) -> Self {
        val.0.into()
    }
}

impl<const N: usize> TryFrom<String<N>> for Locator<N> {
    type Error = ZError;

    fn try_from(s: String<N>) -> Result<Self, Self::Error> {
        let ep = EndPoint::try_from(s)?;
        Ok(ep.into())
    }
}

impl<const N: usize> FromStr for Locator<N> {
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))?)
    }
}

impl<const N: usize> fmt::Display for Locator<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl<const N: usize> fmt::Debug for Locator<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
