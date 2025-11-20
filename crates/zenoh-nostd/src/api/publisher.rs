use zenoh_proto::{
    Encoding, WireExpr, keyexpr,
    network::{NetworkBody, NodeId, QoS, push::Push},
    zenoh::{PushBody, put::Put},
};

use crate::{SessionDriver, platform::Platform};

pub struct ZPublisher<T: Platform + 'static> {
    ke: &'static keyexpr,
    driver: &'static SessionDriver<T>,
}

impl<T: Platform + 'static> ZPublisher<T> {
    pub(crate) fn new(ke: &'static keyexpr, driver: &'static SessionDriver<T>) -> Self {
        Self { ke, driver }
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn put(&self, bytes: &[u8]) -> zenoh_proto::ZResult<()> {
        let msg = NetworkBody::Push(Push {
            wire_expr: WireExpr::from(self.ke),
            qos: QoS::DEFAULT,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::empty(),
                sinfo: None,
                attachment: None,
                payload: bytes,
            }),
        });

        self.driver.send(msg).await
    }
}
