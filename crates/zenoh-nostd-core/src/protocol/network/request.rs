use uhlc::Timestamp;

#[cfg(test)]
use crate::{ZWriter, ZWriterExt};
#[cfg(test)]
use rand::{
    Rng,
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

use crate::{
    ZStruct,
    network::{Budget, Mapping, NodeId, QoS, QueryTarget, Timeout},
    zenoh::RequestBody,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1c")]
pub struct Request<'a> {
    pub id: u32,

    // --- WireExpr TODO: flatten a WireExpr struct or make a ZStruct accepts args ---
    pub scope: u16,
    #[zenoh(header = M)]
    pub mapping: Mapping,
    #[zenoh(presence = header(N), default = "", size = prefixed)]
    pub suffix: &'a str,

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
    pub timeout: Option<Timeout>,

    // --- Body ---
    #[zenoh(size = remain)]
    pub payload: RequestBody<'a>,
}

impl<'a> Request<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let id = thread_rng().r#gen();
        let scope = thread_rng().r#gen();
        let mapping = Mapping::rand();

        let suffix = if thread_rng().gen_bool(0.5) {
            let suffix =
                Alphanumeric.sample_string(&mut thread_rng(), thread_rng().gen_range(1..16));
            w.write_str(&suffix).unwrap()
        } else {
            ""
        };

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

        let budget = thread_rng().gen_bool(0.5).then_some(Budget::rand(w));
        let timeout = thread_rng().gen_bool(0.5).then_some(Timeout::rand(w));

        let payload = RequestBody::rand(w);

        Self {
            id,
            scope,
            mapping,
            suffix,
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
