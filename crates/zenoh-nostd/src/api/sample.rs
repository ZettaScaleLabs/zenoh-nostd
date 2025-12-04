use core::str::FromStr;

use heapless::{String, Vec};
use zenoh_proto::keyexpr;

pub struct Sample<'a> {
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> Sample<'a> {
    pub(crate) fn new(keyexpr: &'a keyexpr, payload: &'a [u8]) -> Sample<'a> {
        Sample { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &'a keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<Sample<'_>>
    for OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::Error;

    fn try_from(value: Sample<'_>) -> Result<Self, Self::Error> {
        Ok(OwnedSample::new(
            String::from_str(value.keyexpr.as_str())
                .map_err(|_| crate::CollectionError::CollectionIsFull)?,
            Vec::from_slice(value.payload).map_err(|_| crate::CollectionError::CollectionIsFull)?,
        ))
    }
}

#[derive(Debug)]
pub struct OwnedSample<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    keyexpr: String<MAX_KEYEXPR>,
    payload: Vec<u8, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub(crate) fn new(
        keyexpr: String<MAX_KEYEXPR>,
        payload: Vec<u8, MAX_PAYLOAD>,
    ) -> OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD> {
        OwnedSample { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.keyexpr.as_str())
    }

    pub fn payload(&self) -> &'_ [u8] {
        &self.payload
    }
}
