use zenoh_proto::{CollectionError, keyexpr};

use crate::{OwnedSample, api::Sample};

#[derive(Debug)]
pub enum Response<'a> {
    Ok(Sample<'a>),
    Err(Sample<'a>),
}

impl<'a> Response<'a> {
    pub fn ok(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self::Ok(Sample::new(ke, payload))
    }

    pub fn err(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self::Err(Sample::new(ke, payload))
    }
}

#[derive(Debug)]
pub enum OwnedResponse<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    Ok(OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>),
    Err(OwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>),
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> OwnedResponse<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub fn as_ref(&self) -> Response<'_> {
        match self {
            Self::Ok(sample) => Response::Ok(sample.as_ref()),
            Self::Err(sample) => Response::Err(sample.as_ref()),
        }
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<&Response<'_>>
    for OwnedResponse<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = CollectionError;

    fn try_from(value: &Response<'_>) -> Result<Self, Self::Error> {
        match value {
            Response::Ok(sample) => Ok(Self::Ok(sample.try_into()?)),
            Response::Err(sample) => Ok(Self::Err(sample.try_into()?)),
        }
    }
}
