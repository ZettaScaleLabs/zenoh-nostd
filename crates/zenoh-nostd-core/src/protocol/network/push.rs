use uhlc::Timestamp;

#[cfg(test)]
use crate::ZWriter;
#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{
    ZStruct,
    network::{NodeId, QoS},
    wire_expr::WireExpr,
    zenoh::PushBody,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1d")]
pub struct Push<'a> {
    #[zenoh(flatten)]
    pub wire_expr: WireExpr<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,

    // --- Body ---
    #[zenoh(size = remain)]
    pub payload: PushBody<'a>,
}

impl<'a> Push<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
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

        let payload = PushBody::rand(w);

        Self {
            wire_expr,
            qos,
            timestamp,
            nodeid,
            payload,
        }
    }
}
