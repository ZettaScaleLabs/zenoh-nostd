use std::str::FromStr;

use zenoh_proto::{
    fields::{ConsolidationMode, Encoding, WireExpr},
    keyexpr,
    msgs::{Err, PushBody, Put, Reply, Response, ResponseBody, ResponseFinal},
    zerror::CollectionError,
};

use crate::{
    ZConfig,
    api::{callbacks::ZCallbacks, driver::Driver, resources::SessionResources},
};

pub struct Query<'a, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,
    resources: &'a SessionResources<'res, Config>,
    rid: u32,
    ke: &'a keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
}

impl<'a, 'res, Config> Query<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'a Driver<'res, Config>,
        resources: &'a SessionResources<'res, Config>,
        rid: u32,
        ke: &'a keyexpr,
        parameters: Option<&'a str>,
        payload: Option<&'a [u8]>,
    ) -> Self {
        Self {
            driver,
            resources,
            rid,
            ke,
            parameters,
            payload,
        }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload
    }

    pub async fn reply(&self, ke: &keyexpr, payload: &[u8]) -> crate::ZResult<()> {
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

    pub async fn err(&self, ke: &keyexpr, payload: &[u8]) -> crate::ZResult<()> {
        let wke = WireExpr::from(ke);

        let response = Response {
            rid: self.rid,
            wire_expr: wke,
            payload: ResponseBody::Err(Err {
                encoding: Encoding::default(),
                payload,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(response).await
    }

    pub async fn finalize(&self) -> crate::ZResult<()> {
        let mut queryable_cb = self.resources.queryable_callbacks.lock().await;
        if queryable_cb.decrease(self.rid) {
            let response = ResponseFinal {
                rid: self.rid,
                ..Default::default()
            };

            self.driver.send(response).await?;
        }

        Ok(())
    }
}

pub struct OwnedQuery<
    Config,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> where
    Config: ZConfig,
{
    driver: &'static Driver<'static, Config>,
    resources: &'static SessionResources<'static, Config>,
    rid: u32,
    ke: heapless::String<MAX_KEYEXPR>,
    parameters: Option<heapless::String<MAX_PARAMETERS>>,
    payload: Option<heapless::Vec<u8, MAX_PAYLOAD>>,
}

impl<Config, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize>
    OwnedQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZConfig,
{
    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.ke.as_str())
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters.as_ref().map(|p| p.as_str())
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_ref().map(|p| p.as_slice())
    }

    pub async fn reply(&self, ke: &keyexpr, payload: &[u8]) -> crate::ZResult<()> {
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

    pub async fn err(&self, ke: &keyexpr, payload: &[u8]) -> crate::ZResult<()> {
        let wke = WireExpr::from(ke);

        let response = Response {
            rid: self.rid,
            wire_expr: wke,
            payload: ResponseBody::Err(Err {
                encoding: Encoding::default(),
                payload,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(response).await
    }

    pub async fn finalize(&self) -> crate::ZResult<()> {
        let mut queryable_cb = self.resources.queryable_callbacks.lock().await;
        if queryable_cb.decrease(self.rid) {
            let response = ResponseFinal {
                rid: self.rid,
                ..Default::default()
            };

            self.driver.send(response).await?;
        }

        Ok(())
    }
}

impl<'a, Config, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize>
    TryFrom<(
        &Query<'a, 'static, Config>,
        &'static Driver<'static, Config>,
        &'static SessionResources<'static, Config>,
    )> for OwnedQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZConfig,
{
    type Error = CollectionError;

    fn try_from(
        value: (
            &Query<'a, 'static, Config>,
            &'static Driver<'static, Config>,
            &'static SessionResources<'static, Config>,
        ),
    ) -> Result<Self, Self::Error> {
        let (value, driver, resources) = value;

        Ok(Self {
            driver,
            resources,
            rid: value.rid,
            ke: heapless::String::from_str(value.keyexpr().as_str())
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            parameters: value
                .parameters
                .map(|p| heapless::String::from_str(p))
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            payload: value
                .payload
                .map(|p| heapless::Vec::from_slice(p))
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
        })
    }
}
