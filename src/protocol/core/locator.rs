use core::{convert::TryFrom, fmt, hash::Hash, str::FromStr};

use heapless::String;

use crate::{
    protocol::{
        ZCodecError,
        zcodec::{decode_str, encode_str},
    },
    result::ZResult,
    zbuf::ZBufWriter,
};

use super::endpoint::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Locator<const N: usize>(pub(super) EndPoint<N>);

impl<const N: usize> Locator<N> {
    pub fn new<A, B>(
        protocol: A,
        address: B,
    ) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError>
    where
        A: AsRef<str>,
        B: AsRef<str>,
    {
        let ep = EndPoint::new(protocol, address)?;
        Ok(Self(ep))
    }

    pub fn protocol(&self) -> Protocol<'_> {
        self.0.protocol()
    }

    pub fn address(&self) -> Address<'_> {
        self.0.address()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_endpoint(&self) -> EndPoint<N> {
        self.0.clone()
    }

    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        encode_str(true, self.as_str(), writer)
    }

    pub fn decode(reader: &mut &[u8]) -> ZResult<Self, ZCodecError> {
        let s = decode_str(None, reader)?;
        Ok(Self::try_from(s)?)
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        EndPoint::rand().into()
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
    type Error = crate::protocol::ZProtocolError;

    fn try_from(s: &str) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
        Self::try_from(String::from_str(s).map_err(|_| crate::protocol::ZProtocolError::Invalid)?)
    }
}

impl<const N: usize> From<Locator<N>> for String<N> {
    fn from(val: Locator<N>) -> Self {
        val.0.into()
    }
}

impl<const N: usize> TryFrom<String<N>> for Locator<N> {
    type Error = crate::protocol::ZProtocolError;

    fn try_from(s: String<N>) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
        let ep = EndPoint::try_from(s)?;
        Ok(ep.into())
    }
}

impl<const N: usize> FromStr for Locator<N> {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from_str(s).map_err(|_| crate::protocol::ZProtocolError::Invalid)?)
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
