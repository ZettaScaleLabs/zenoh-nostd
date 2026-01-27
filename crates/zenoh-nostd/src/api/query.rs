use core::str::FromStr;

use zenoh_proto::{
    CollectionError, SessionError,
    fields::{ConsolidationMode, Encoding, WireExpr},
    keyexpr,
    msgs::{Err, PushBody, Put, Reply, Response, ResponseBody, ResponseFinal},
};

use crate::{api::session::Session, config::ZSessionConfig};

pub struct QueryableQuery<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'session Session<'ext, 'res, Config>,
    rid: u32,
    ke: &'parameters keyexpr,
    parameters: Option<&'parameters str>,
    payload: Option<&'parameters [u8]>,
}

impl<'parameters, 'session, 'ext, 'res, Config>
    QueryableQuery<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(
        session: &'session Session<'ext, 'res, Config>,
        rid: u32,
        ke: &'parameters keyexpr,
        parameters: Option<&'parameters str>,
        payload: Option<&'parameters [u8]>,
    ) -> Self {
        Self {
            session,
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

    pub async fn reply(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        todo!()
        // let wke = WireExpr::from(ke);

        // let response = Response {
        //     rid: self.rid,
        //     wire_expr: wke,
        //     payload: ResponseBody::Reply(Reply {
        //         consolidation: ConsolidationMode::None,
        //         payload: PushBody::Put(Put {
        //             payload,
        //             ..Default::default()
        //         }),
        //     }),
        //     ..Default::default()
        // };

        // self.session.driver.tx().await.send(response).await
    }

    pub async fn err(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        todo!()
        // let wke = WireExpr::from(ke);

        // let response = Response {
        //     rid: self.rid,
        //     wire_expr: wke,
        //     payload: ResponseBody::Err(Err {
        //         encoding: Encoding::default(),
        //         payload,
        //         ..Default::default()
        //     }),
        //     ..Default::default()
        // };

        // self.driver.send(response).await
    }

    pub async fn finalize(&self) -> core::result::Result<(), SessionError> {
        todo!()
        // let mut queryable_cb = self.resources.queryable_callbacks.lock().await;
        // if queryable_cb.decrease(self.rid) {
        //     let response = ResponseFinal {
        //         rid: self.rid,
        //         ..Default::default()
        //     };

        //     self.driver.send(response).await?;
        // }

        // Ok(())
    }
}

pub struct OwnedQueryableQuery<
    Config,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> where
    Config: ZSessionConfig + 'static,
{
    session: &'static Session<'static, 'static, Config>,
    rid: u32,
    ke: heapless::String<MAX_KEYEXPR>,
    parameters: Option<heapless::String<MAX_PARAMETERS>>,
    payload: Option<heapless::Vec<u8, MAX_PAYLOAD>>,
}

impl<Config, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize>
    OwnedQueryableQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZSessionConfig,
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

    pub async fn reply(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        todo!()
        // let wke = WireExpr::from(ke);

        // let response = Response {
        //     rid: self.rid,
        //     wire_expr: wke,
        //     payload: ResponseBody::Reply(Reply {
        //         consolidation: ConsolidationMode::None,
        //         payload: PushBody::Put(Put {
        //             payload,
        //             ..Default::default()
        //         }),
        //     }),
        //     ..Default::default()
        // };

        // self.driver.send(response).await
    }

    pub async fn err(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        todo!()
        // let wke = WireExpr::from(ke);

        // let response = Response {
        //     rid: self.rid,
        //     wire_expr: wke,
        //     payload: ResponseBody::Err(Err {
        //         encoding: Encoding::default(),
        //         payload,
        //         ..Default::default()
        //     }),
        //     ..Default::default()
        // };

        // self.driver.send(response).await
    }

    pub async fn finalize(&self) -> core::result::Result<(), SessionError> {
        todo!()
        // let mut queryable_cb = self.resources.queryable_callbacks.lock().await;
        // if queryable_cb.decrease(self.rid) {
        //     let response = ResponseFinal {
        //         rid: self.rid,
        //         ..Default::default()
        //     };

        //     self.driver.send(response).await?;
        // }

        // Ok(())
    }
}

impl<
    'parameters,
    Config,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> TryFrom<&QueryableQuery<'parameters, 'static, 'static, 'static, Config>>
    for OwnedQueryableQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZSessionConfig,
{
    type Error = CollectionError;

    fn try_from(
        value: &QueryableQuery<'parameters, 'static, 'static, 'static, Config>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            session: value.session,
            rid: value.rid,
            ke: heapless::String::from_str(value.keyexpr().as_str())
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            parameters: value
                .parameters
                .map(heapless::String::from_str)
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            payload: value
                .payload
                .map(heapless::Vec::from_slice)
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
        })
    }
}
