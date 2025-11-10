use core::time::Duration;

use rand::{Rng, thread_rng};

use crate::{
    ZEncode, ZWriter,
    network::{declare::*, interest::*, push::*, request::*, response::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    Duration,
    QueryableInfo
);
crate::roundtrips!(
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
        let mut rng = thread_rng();
        for _ in 1..16 {
            let choice = rng.gen_range(0..6);
            match choice {
                0 => msgs.push_back(NetworkBody::Push(Push::rand(&mut rand_writer))),
                1 => msgs.push_back(NetworkBody::Request(Request::rand(&mut rand_writer))),
                2 => msgs.push_back(NetworkBody::Response(Response::rand(&mut rand_writer))),
                3 => msgs.push_back(NetworkBody::ResponseFinal(ResponseFinal::rand(
                    &mut rand_writer,
                ))),
                4 => msgs.push_back(NetworkBody::Interest(Interest::rand(&mut rand_writer))),
                5 => msgs.push_back(NetworkBody::Declare(Declare::rand(&mut rand_writer))),
                _ => unreachable!(),
            }
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut writer = data.as_mut_slice();
    let start = writer.len();
    for msg in &messages {
        match msg {
            NetworkBody::Push(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
            NetworkBody::Request(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
            NetworkBody::Response(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
            NetworkBody::ResponseFinal(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
            NetworkBody::Interest(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
            NetworkBody::Declare(m) => <_ as ZEncode>::z_encode(m, &mut writer).unwrap(),
        }
    }
    let len = start - writer.len();

    let mut reader = &data[..len];
    let iter = NetworkBodyIter::new(&mut reader);

    for expected in iter {
        let actual = messages.pop_front().unwrap();
        assert_eq!(actual, expected);
    }
}
