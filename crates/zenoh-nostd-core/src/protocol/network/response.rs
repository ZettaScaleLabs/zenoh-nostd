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
    network::{Mapping, QoS},
    zenoh::{EntityGlobalId, ResponseBody},
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1b")]
pub struct Response<'a> {
    pub rid: u32,

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
    #[zenoh(ext = 0x3)]
    pub respid: Option<EntityGlobalId>,

    // --- Body ---
    #[zenoh(size = remain)]
    pub payload: ResponseBody<'a>,
}

impl<'a> Response<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let rid = thread_rng().r#gen();
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

        let respid = thread_rng()
            .gen_bool(0.5)
            .then_some(EntityGlobalId::rand(w));

        let payload = ResponseBody::rand(w);

        Self {
            rid,
            scope,
            mapping,
            suffix,
            qos,
            timestamp,
            respid,
            payload,
        }
    }
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x1a")]
pub struct ResponseFinal {
    pub rid: u32,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
}

impl ResponseFinal {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter) -> Self {
        let rid = thread_rng().r#gen();

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

        Self {
            rid,
            qos,
            timestamp,
        }
    }
}
