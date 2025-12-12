//! Query and HeaplessQuery types

use core::convert::TryFrom;
use heapless::{String, Vec};
use higher_kinded_types::ForLt;
use zenoh_proto::{
    fields::{ConsolidationMode, WireExpr},
    keyexpr,
    msgs::{Err, PushBody, Put, Reply, Response, ResponseBody, ResponseFinal},
};

use crate::api::{ZConfig, driver::Driver};

pub(crate) type QueryRef<Config> = ForLt!(<'a> = &'a Query<'a, Config>);

pub struct Query<'a, Config>
where
    Config: ZConfig,
{
    driver: &'static Driver<'static, Config>,
    rid: u32,
    keyexpr: &'a keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
}

impl<'a, Config> Query<'a, Config>
where
    Config: ZConfig,
{
    pub fn new(
        driver: &'static Driver<'static, Config>,
        rid: u32,
        keyexpr: &'a keyexpr,
        parameters: Option<&'a str>,
        payload: Option<&'a [u8]>,
    ) -> Self {
        Self {
            driver,
            rid,
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

pub struct HeaplessQuery<
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
    Config,
> where
    Config: ZConfig,
{
    driver: &'static Driver<'static, Config>,
    rid: u32,
    keyexpr: String<MAX_KEYEXPR>,
    parameters: Option<String<MAX_PARAMETERS>>,
    payload: Option<Vec<u8, MAX_PAYLOAD>>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize, Config>
    HeaplessQuery<MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD, Config>
where
    Config: ZConfig,
{
    pub fn new(
        driver: &'static Driver<'static, Config>,
        rid: u32,
        keyexpr: &keyexpr,
        parameters: Option<&str>,
        payload: Option<&[u8]>,
    ) -> Result<Self, crate::CollectionError> {
        let mut ke_str = String::<MAX_KEYEXPR>::new();
        ke_str
            .push_str(keyexpr.as_str())
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        let parameters = if let Some(p) = parameters {
            let mut p_str = String::<MAX_PARAMETERS>::new();
            p_str
                .push_str(p)
                .map_err(|_| crate::CollectionError::CollectionIsFull)?;
            Some(p_str)
        } else {
            None
        };

        let payload = if let Some(bytes) = payload {
            let mut p = Vec::<u8, MAX_PAYLOAD>::new();
            p.extend_from_slice(bytes)
                .map_err(|_| crate::CollectionError::CollectionIsFull)?;
            Some(p)
        } else {
            None
        };

        Ok(Self {
            driver,
            rid,
            keyexpr: ke_str,
            parameters,
            payload,
        })
    }

    /// Reconstructs a valid `keyexpr` via unsafe `keyexpr::new()`.
    pub fn keyexpr(&self) -> &keyexpr {
        unsafe { keyexpr::new(self.keyexpr.as_str()).unwrap_unchecked() }
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters.as_deref()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
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
                payload,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(response).await
    }

    pub async fn finalize(self) -> crate::ZResult<()> {
        let response = ResponseFinal {
            rid: self.rid,
            ..Default::default()
        };

        self.driver.send(response).await
    }
}

impl<'a, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize, Config>
    TryFrom<&Query<'a, Config>> for HeaplessQuery<MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD, Config>
where
    Config: ZConfig,
{
    type Error = crate::CollectionError;

    fn try_from(q: &Query<'a, Config>) -> Result<Self, Self::Error> {
        HeaplessQuery::new(q.driver, q.rid, q.keyexpr, q.parameters, q.payload)
    }
}
