use rand::{Rng, thread_rng};

use crate::{
    ZWriter,
    core::{encoding::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, core, Timestamp);
crate::roundtrips!(core, ZenohIdProto, Timestamp, Reliability, Encoding);

trait RandTimestamp {
    fn rand(w: &mut ZWriter) -> Self;
}

impl RandTimestamp for Timestamp {
    fn rand(_: &mut ZWriter) -> Self {
        let time = uhlc::NTP64(thread_rng().r#gen());
        let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
        Timestamp::new(time, id)
    }
}
