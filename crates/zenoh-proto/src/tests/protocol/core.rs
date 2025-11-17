use rand::{Rng, thread_rng};

use crate::{ZWriter, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

super::roundtrips!(ext, core, Timestamp);
super::roundtrips!(core, ZenohIdProto, Timestamp, Encoding);

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
