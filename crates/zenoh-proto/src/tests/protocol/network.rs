use core::time::Duration;

use rand::{Rng, thread_rng};

use crate::{
    ZEncode, ZWriter,
    network::{declare::*, interest::*, push::*, request::*, response::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

super::roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    Duration,
    QueryableInfo
);
super::roundtrips!(
    network,
    DeclareKeyExpr,
    UndeclareKeyExpr,
    DeclareSubscriber,
    UndeclareSubscriber,
    DeclareQueryable,
    UndeclareQueryable,
    DeclareToken,
    UndeclareToken,
    DeclareFinal,
    DeclareBody,
    Declare,
    Interest,
    Push,
    Request,
    Response,
    ResponseFinal,
);

trait RandDuration {
    fn rand(w: &mut ZWriter) -> Self;
}

impl RandDuration for Duration {
    fn rand(_: &mut ZWriter) -> Self {
        Duration::from_millis(thread_rng().gen_range(0..10_000))
    }
}

#[test]
fn network_stream() {
    extern crate std;
    use std::collections::VecDeque;

    let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut rand_writer = rand_data.as_mut_slice();

    let mut messages = {
        let mut msgs = VecDeque::new();
        for _ in 1..thread_rng().gen_range(1..16) {
            msgs.push_back(NetworkBody::rand(&mut rand_writer));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut writer = data.as_mut_slice();
    let start = writer.len();
    for msg in &messages {
        <_ as ZEncode>::z_encode(msg, &mut writer).unwrap();
    }
    let len = start - writer.len();

    let mut reader = &data[..len];
    let iter = NetworkBatch::new(&mut reader);

    for expected in iter {
        let actual = messages.pop_front().unwrap();
        assert_eq!(actual, expected.unwrap());
    }
}
