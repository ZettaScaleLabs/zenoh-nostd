use std::str::FromStr;

use zenoh_proto::{keyexpr, zerror::CollectionError};

#[derive(Debug)]
pub struct Sample<'a> {
    ke: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> Sample<'a> {
    pub fn new(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self { ke, payload }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}

#[derive(Debug)]
pub struct OwnedSample<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    ke: heapless::String<MAX_KEYEXPR>,
    payload: heapless::Vec<u8, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.ke.as_str())
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    pub fn as_ref(&self) -> Sample<'_> {
        Sample {
            ke: self.keyexpr(),
            payload: self.payload(),
        }
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<&Sample<'_>>
    for OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = CollectionError;

    fn try_from(value: &Sample<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            ke: heapless::String::from_str(value.keyexpr().as_str())
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            payload: heapless::Vec::from_slice(value.payload())
                .map_err(|_| CollectionError::CollectionTooSmall)?,
        })
    }
}
