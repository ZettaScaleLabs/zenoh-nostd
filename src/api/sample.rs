use core::str::FromStr;

use heapless::{String, Vec};

use crate::{
    protocol::keyexpr::borrowed::keyexpr,
    result::{ZError, ZResult},
    zbuf::ZBuf,
};

pub struct ZSample<'a> {
    keyexpr: &'a keyexpr,
    payload: ZBuf<'a>,
}

impl<'a> ZSample<'a> {
    pub(crate) fn new(keyexpr: &'a keyexpr, payload: ZBuf<'a>) -> ZSample<'a> {
        ZSample { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &'a keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> ZBuf<'a> {
        self.payload
    }

    pub(crate) fn into_owned<const KE: usize, const PL: usize>(
        self,
    ) -> ZResult<ZOwnedSample<KE, PL>> {
        Ok(ZOwnedSample::new(
            String::from_str(self.keyexpr.as_str()).map_err(|_| ZError::CapacityExceeded)?,
            Vec::from_slice(self.payload).map_err(|_| ZError::CapacityExceeded)?,
        ))
    }
}

#[derive(Debug, Clone)]
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

    pub fn payload(&self) -> ZBuf<'_> {
        &self.payload
    }
}
