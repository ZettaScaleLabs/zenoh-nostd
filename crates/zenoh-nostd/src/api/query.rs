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

pub struct ZOwnedQuery<
    T: Platform + 'static,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> {
    rid: u32,
    driver: &'static SessionDriver<T>,
    payload: Vec<u8, MAX_PAYLOAD>,
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
        payload: Vec<u8, MAX_PAYLOAD>,
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

    pub fn payload(&self) -> &[u8] {
        &self.payload
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
