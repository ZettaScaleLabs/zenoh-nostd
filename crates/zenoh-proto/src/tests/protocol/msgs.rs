use rand::{Rng, thread_rng};

use crate::{exts::*, fields::*, msgs::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

super::roundtrips!(ext, zenoh, EntityGlobalId, SourceInfo, Value, Attachment);
super::roundtrips!(zenoh, Err, Put, Query, Reply,);

super::roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
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
    Declare,
    Interest,
    InterestFinal,
    Push,
    Request,
    Response,
    ResponseFinal,
);

super::roundtrips!(ext, transport, Auth, Patch);
super::roundtrips!(
    transport,
    Close,
    FrameHeader,
    InitSyn,
    InitAck,
    KeepAlive,
    OpenSyn,
    OpenAck
);

#[derive(ZEnum, Debug, PartialEq)]
pub enum FrameBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    InterestFinal(InterestFinal),
    Declare(Declare<'a>),
}

impl Framed for FrameBody<'_> {}

impl<'a> FrameBody<'a> {
    pub(crate) fn rand(w: &mut impl crate::ZStore<'a>) -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let choices = [
            Push::ID,
            Request::ID,
            Response::ID,
            ResponseFinal::ID,
            Interest::ID,
            Declare::ID,
        ];

        match *choices.choose(&mut rng).unwrap() {
            Push::ID => FrameBody::Push(Push::rand(w)),
            Request::ID => FrameBody::Request(Request::rand(w)),
            Response::ID => FrameBody::Response(Response::rand(w)),
            ResponseFinal::ID => FrameBody::ResponseFinal(ResponseFinal::rand(w)),
            Interest::ID => {
                if rng.gen_bool(0.5) {
                    FrameBody::Interest(Interest::rand(w))
                } else {
                    FrameBody::InterestFinal(InterestFinal::rand(w))
                }
            }
            Declare::ID => FrameBody::Declare(Declare::rand(w)),
            _ => unreachable!(),
        }
    }

    pub fn is(&self, x: &ZMessage<'a>) -> bool {
        match (self, x) {
            (FrameBody::Push(x), ZMessage::Push { body: y, .. }) => x == y,
            (FrameBody::Request(x), ZMessage::Request { body: y, .. }) => x == y,
            (FrameBody::Response(x), ZMessage::Response { body: y, .. }) => x == y,
            (FrameBody::ResponseFinal(x), ZMessage::ResponseFinal { body: y, .. }) => x == y,
            (FrameBody::Interest(x), ZMessage::Interest { body: y, .. }) => x == y,
            (FrameBody::InterestFinal(x), ZMessage::InterestFinal { body: y, .. }) => x == y,
            (FrameBody::Declare(x), ZMessage::Declare { body: y, .. }) => x == y,
            _ => false,
        }
    }
}

#[test]
fn network_stream() {
    extern crate std;
    use std::collections::VecDeque;

    let mut rand = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut rw = rand.as_mut_slice();

    let mut messages = {
        let mut msgs = VecDeque::new();
        for _ in 1..thread_rng().gen_range(1..16) {
            msgs.push_back(FrameBody::rand(&mut rw));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut batch = ZBatchWriter::new(&mut data[..], 0);

    for msg in &messages {
        batch
            .frame(msg, Reliability::default(), QoS::default())
            .unwrap();
    }

    let (_, len) = batch.finalize();
    let batch = ZBatchReader::new(&data[..len]);

    for msg in batch {
        let actual = messages.pop_front().unwrap();
        assert_eq!(true, actual.is(&msg));
    }
}

#[test]
fn transport_stream() {
    extern crate std;
    use std::collections::VecDeque;

    let mut rand = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut rw = rand.as_mut_slice();

    let mut messages = {
        let mut msgs = VecDeque::new();
        for _ in 1..thread_rng().gen_range(1..16) {
            msgs.push_back((Reliability::rand(&mut rw), FrameBody::rand(&mut rw)));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut batch = ZBatchWriter::new(&mut data[..], 0);

    for (r, msg) in &messages {
        batch.frame(msg, *r, QoS::default()).unwrap();
    }

    batch.unframe(&KeepAlive {}).unwrap();

    let (_, len) = batch.finalize();
    let batch = ZBatchReader::new(&data[..len]);

    let mut got_keepalive = false;
    for msg in batch {
        if let Some((_, actual)) = messages.pop_front() {
            if actual.is(&msg) {
                continue;
            } else {
                panic!("Frame message did not match");
            }
        }

        match msg {
            ZMessage::KeepAlive(_) => {
                got_keepalive = true;
            }
            _ => panic!("First messages should be Frames, and last a KeepAlive"),
        }
    }

    assert!(messages.is_empty());
    assert!(got_keepalive);
}
