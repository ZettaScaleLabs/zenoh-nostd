use zenoh_proto::{
    Encoding, WireExpr, keyexpr,
    network::{NodeId, QoS, push::Push},
    zenoh::{PushBody, put::Put},
};

use crate::{
    Session,
    event::{Event, EventInner},
};

impl Session {
    pub fn put<'a>(&self, ke: &'static keyexpr, payload: &'a [u8]) -> Event<'a> {
        Event {
            inner: EventInner::Push(Push {
                wire_expr: WireExpr::from(ke),
                qos: QoS::DEFAULT,
                timestamp: None,
                nodeid: NodeId::DEFAULT,
                payload: PushBody::Put(Put {
                    timestamp: None,
                    encoding: Encoding::DEFAULT,
                    sinfo: None,
                    attachment: None,
                    payload,
                }),
            }),
        }
    }
}
