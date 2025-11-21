use crate::{ZOwnedSample, ZSample};

pub enum ZReply<'a> {
    Ok(ZSample<'a>),
    Err(ZSample<'a>),
}

#[derive(Debug)]
pub enum ZOwnedReply<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    Ok(ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>),
    Err(ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>),
}

impl ZReply<'_> {
    pub fn into_owned<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize>(
        self,
    ) -> crate::ZResult<ZOwnedReply<MAX_KEYEXPR, MAX_PAYLOAD>> {
        match self {
            ZReply::Ok(sample) => Ok(ZOwnedReply::Ok(sample.into_owned()?)),
            ZReply::Err(sample) => Ok(ZOwnedReply::Err(sample.into_owned()?)),
        }
    }
}
