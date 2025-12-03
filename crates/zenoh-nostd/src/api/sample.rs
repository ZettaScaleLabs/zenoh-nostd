use ::core::str::FromStr;

use heapless::{String, Vec};
use zenoh_proto::keyexpr;

pub struct ZSample<'a> {
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> ZSample<'a> {
    pub(crate) fn new(keyexpr: &'a keyexpr, payload: &'a [u8]) -> ZSample<'a> {
        ZSample { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &'a keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<ZSample<'_>>
    for ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::ZError;

    fn try_from(value: ZSample<'_>) -> Result<Self, Self::Error> {
        Ok(ZOwnedSample::new(
            String::from_str(value.keyexpr.as_str())
                .map_err(|_| crate::ZError::CapacityExceeded)?,
            Vec::from_slice(value.payload).map_err(|_| crate::ZError::CapacityExceeded)?,
        ))
    }
}

#[derive(Debug)]
pub struct ZOwnedSample<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    keyexpr: String<MAX_KEYEXPR>,
    payload: Vec<u8, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub(crate) fn new(
        keyexpr: String<MAX_KEYEXPR>,
        payload: Vec<u8, MAX_PAYLOAD>,
    ) -> ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD> {
        ZOwnedSample { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.keyexpr.as_str())
    }

    pub fn payload(&self) -> &'_ [u8] {
        &self.payload
    }
}
