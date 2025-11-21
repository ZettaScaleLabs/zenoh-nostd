use embassy_time::{Duration, Instant};
use zenoh_proto::{
    Encoding, WireExpr, ZResult, keyexpr,
    network::{NetworkBody, NodeId, QoS, QueryTarget, request::Request},
    zenoh::{ConsolidationMode, RequestBody, Value, query::Query},
};

use crate::{Session, ZRepliesCallback, ZReply, platform::Platform, session::NEXT_ID};

pub struct GetBuilder<'a, T: Platform + 'static> {
    session: &'a Session<T>,
    ke: &'static keyexpr,
    callback: fn(&ZReply<'_>),

    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
    timeout: Option<Duration>,
}

impl<'a, T: Platform + 'static> GetBuilder<'a, T> {
    pub(crate) fn new(
        session: &'a Session<T>,
        ke: &'static keyexpr,
        callback: fn(&ZReply<'_>),
    ) -> Self {
        Self {
            session,
            ke,
            callback,

            parameters: None,
            payload: None,
            timeout: None,
        }
    }

    pub fn parameters(mut self, parameters: &'a str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn payload(mut self, payload: &'a [u8]) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn send(self) -> ZResult<()> {
        let wke = WireExpr::from(self.ke);

        let mut id = NEXT_ID.lock().await;
        *id += 1;
        let id = *id;

        let expiration = Instant::now()
            .checked_add(self.timeout.unwrap_or(Duration::from_secs(5)))
            .unwrap();

        self.session
            .driver
            .as_ref()
            .unwrap()
            .register_query_callback(
                id,
                self.ke,
                ZRepliesCallback::new(self.callback, expiration),
            )
            .await?;

        let msg = NetworkBody::Request(Request {
            id,
            wire_expr: wke,
            payload: RequestBody::Query(Query {
                consolidation: ConsolidationMode::None,
                parameters: self.parameters.unwrap_or_default(),
                body: self.payload.map(|p| Value {
                    encoding: Encoding::DEFAULT,
                    payload: p,
                }),
                attachment: None,
                sinfo: None,
            }),
            qos: QoS::DEFAULT,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            budget: None,
            timeout: None,
            target: QueryTarget::DEFAULT,
        });

        self.session.driver.as_ref().unwrap().send(msg).await?;

        Ok(())
    }
}
