use core::str::FromStr;

use heapless::{String, Vec};
use zenoh_proto::{
    Encoding, WireExpr, ZResult, keyexpr,
    network::{
        NetworkBody, QoS,
        response::{Response, ResponseFinal},
    },
    zenoh::{ConsolidationMode, PushBody, ResponseBody, err::Err, put::Put, reply::Reply},
};

use crate::{SessionDriver, platform::Platform};

pub struct ZQuery<'a, T: Platform + 'static> {
    rid: u32,
    driver: &'static SessionDriver<T>,
    keyexpr: &'a keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
}

impl<'a, T: Platform + 'static> ZQuery<'a, T> {
    pub(crate) fn new(
        rid: u32,
        driver: &'static SessionDriver<T>,
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
    ) -> ZResult<ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>> {
        Ok(ZOwnedQuery::new(
            self.rid,
            self.driver,
            String::from_str(self.keyexpr.as_str())
                .map_err(|_| zenoh_proto::ZError::CapacityExceeded)?,
            match self.parameters {
                Some(params) => Some(
                    String::from_str(params).map_err(|_| zenoh_proto::ZError::CapacityExceeded)?,
                ),
                None => None,
            },
            match self.payload {
                Some(payload) => {
                    let mut vec: Vec<u8, MAX_PAYLOAD> = Vec::new();
                    vec.extend_from_slice(payload)
                        .map_err(|_| zenoh_proto::ZError::CapacityExceeded)?;
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

    pub async fn reply(&self, ke: &'static keyexpr, payload: &[u8]) -> ZResult<()> {
        let wke = WireExpr::from(ke);

        let response = NetworkBody::Response(Response {
            rid: self.rid,
            wire_expr: wke,
            qos: QoS::DEFAULT,
            timestamp: None,
            respid: None,
            payload: ResponseBody::Reply(Reply {
                consolidation: ConsolidationMode::None,
                payload: PushBody::Put(Put {
                    payload,
                    attachment: None,
                    encoding: Encoding::DEFAULT,
                    timestamp: None,
                    sinfo: None,
                }),
            }),
        });

        self.driver.send(response).await
    }

    pub async fn err(&self, payload: &[u8]) -> ZResult<()> {
        let response = NetworkBody::Response(Response {
            rid: self.rid,
            wire_expr: WireExpr::from(self.keyexpr()),
            qos: QoS::DEFAULT,
            timestamp: None,
            respid: None,
            payload: ResponseBody::Err(Err {
                encoding: Encoding::DEFAULT,
                sinfo: None,
                payload,
            }),
        });

        self.driver.send(response).await
    }

    pub async fn finalize(&self) -> ZResult<()> {
        let response: NetworkBody<'static> = NetworkBody::ResponseFinal(ResponseFinal {
            rid: self.rid,
            qos: QoS::DEFAULT,
            timestamp: None,
        });

        self.driver.send(response).await
    }
}
