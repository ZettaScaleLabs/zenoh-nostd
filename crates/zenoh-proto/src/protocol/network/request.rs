use core::time::Duration;

use uhlc::Timestamp;

use crate::{
    WireExpr, ZStruct,
    network::{Budget, NodeId, QoS, QueryTarget},
    zenoh::RequestBody,
};

#[cfg(test)]
use {
    crate::ZWriter,
    rand::{Rng, thread_rng},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1c")]
pub struct Request<'a> {
    pub id: u32,

    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,
    #[zenoh(ext = 0x4, default = QueryTarget::DEFAULT, mandatory)]
    pub target: QueryTarget,
    #[zenoh(ext = 0x5)]
    pub budget: Option<Budget>,
    #[zenoh(ext = 0x6)]
    pub timeout: Option<Duration>,

    // --- Body ---
    pub payload: RequestBody<'a>,
}

impl<'a> Request<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let id = thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);

        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let target = if thread_rng().gen_bool(0.5) {
            QueryTarget::rand(w)
        } else {
            QueryTarget::DEFAULT
        };

        trait RandDuration {
            fn rand(w: &mut ZWriter) -> Self;
        }

        impl RandDuration for Duration {
            fn rand(_: &mut ZWriter) -> Self {
                Duration::from_millis(thread_rng().gen_range(0..10_000))
            }
        }

        let budget = thread_rng().gen_bool(0.5).then_some(Budget::rand(w));
        let timeout = thread_rng().gen_bool(0.5).then_some(Duration::rand(w));

        let payload = RequestBody::rand(w);

        Self {
            id,
            wire_expr,
            qos,
            timestamp,
            nodeid,
            target,
            budget,
            timeout,
            payload,
        }
    }
}
