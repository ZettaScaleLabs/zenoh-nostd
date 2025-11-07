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
    network::{Mapping, NodeId, QoS},
    zenoh::PushBody,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1d")]
pub struct Push<'a> {
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

    // --- Body ---
    #[zenoh(size = remain)]
    pub payload: PushBody<'a>,
}

impl<'a> Push<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
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

        let payload = PushBody::rand(w);

        Self {
            scope,
            mapping,
            suffix,
            qos,
            timestamp,
            nodeid,
            payload,
        }
    }
}
