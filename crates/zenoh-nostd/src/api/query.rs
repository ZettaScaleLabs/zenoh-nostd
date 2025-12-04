use core::str::FromStr;

use heapless::{String, Vec};
use zenoh_proto::{fields::*, msgs::*, *};

use crate::{SessionDriver, platform::Platform};

pub struct ZQuery<'a, T: Platform + 'static, TX: AsMut<[u8]> + 'static> {
    rid: u32,
    driver: &'static SessionDriver<T, TX>,
    keyexpr: &'a keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
}

impl<'a, T: Platform + 'static, TX: AsMut<[u8]> + 'static> ZQuery<'a, T, TX> {
    pub(crate) fn new(
        rid: u32,
        driver: &'static SessionDriver<T, TX>,
        keyexpr: &'a keyexpr,
        parameters: Option<&'a str>,
        payload: Option<&'a [u8]>,
    ) -> Self {
        Self {
            rid,
            driver,
            keyexpr,
            parameters,
            payload,
        }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.keyexpr
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload
    }

    pub(crate) fn into_owned<
        const MAX_KEYEXPR: usize,
        const MAX_PARAMETERS: usize,
        const MAX_PAYLOAD: usize,
    >(
        self,
    ) -> crate::ZResult<ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>> {
        Ok(ZOwnedQuery::new(
            self.rid,
            self.driver,
            String::from_str(self.keyexpr.as_str())
                .map_err(|_| zenoh_proto::Error::CapacityExceeded)?,
            match self.parameters {
                Some(params) => Some(
                    String::from_str(params).map_err(|_| zenoh_proto::Error::CapacityExceeded)?,
                ),
                None => None,
            },
            match self.payload {
                Some(payload) => {
                    let mut vec: Vec<u8, MAX_PAYLOAD> = Vec::new();
                    vec.extend_from_slice(payload)
                        .map_err(|_| zenoh_proto::Error::CapacityExceeded)?;
                    Some(vec)
                }
                None => None,
            },
        ))
    }
}

pub struct ZOwnedQuery<
    T: Platform + 'static,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> {
    rid: u32,
    driver: &'static SessionDriver<T>,
    payload: Option<Vec<u8, MAX_PAYLOAD>>,
    parameters: Option<String<MAX_PARAMETERS>>,
    keyexpr: String<MAX_KEYEXPR>,
}

impl<
    T: Platform + 'static,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
{
    pub(crate) fn new(
        rid: u32,
        driver: &'static SessionDriver<T>,
        keyexpr: String<MAX_KEYEXPR>,
        parameters: Option<String<MAX_PARAMETERS>>,
        payload: Option<Vec<u8, MAX_PAYLOAD>>,
    ) -> Self {
        Self {
            rid,
            driver,
            keyexpr,
            parameters,
            payload,
        }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.keyexpr.as_str())
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters.as_deref()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    pub async fn reply(&self, ke: &'static keyexpr, payload: &[u8]) -> crate::ZResult<()> {
        let wke = WireExpr::from(ke);

        let response = Response {
            rid: self.rid,
            wire_expr: wke,
            payload: ResponseBody::Reply(Reply {
                consolidation: ConsolidationMode::None,
                payload: PushBody::Put(Put {
                    payload,
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };

        self.driver.send(response).await
    }

    pub async fn err(&self, payload: &[u8]) -> crate::ZResult<()> {
        let response = Response {
            rid: self.rid,
            wire_expr: WireExpr::from(self.keyexpr()),
            payload: ResponseBody::Err(Err {
                payload,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(response).await
    }

    pub async fn finalize(&self) -> crate::ZResult<()> {
        let response = ResponseFinal {
            rid: self.rid,
            ..Default::default()
        };

        self.driver.send(response).await
    }
}
