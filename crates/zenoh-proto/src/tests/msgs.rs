use rand::{Rng, thread_rng};

use crate::{exts::*, fields::*, msgs::*, *};

macro_rules! roundtrip {
    ($ty:ty) => {{
        let mut rand = [0u8; MAX_PAYLOAD_SIZE];
        let mut data = [0u8; MAX_PAYLOAD_SIZE];

        for _ in 0..NUM_ITER {
            let value = <$ty>::rand(&mut &mut rand[..]);

            let len = $crate::ZLen::z_len(&value);
            $crate::ZEncode::z_encode(&value, &mut &mut data[..]).unwrap();

            let ret = <$ty as $crate::ZDecode>::z_decode(&mut &data[..len]).unwrap();

            assert_eq!(ret, value);
        }

        #[cfg(feature = "alloc")]
        {
            // Because random data generation uses the `ZStoreable` unsafe trait, we need
            // to avoid reallocation during the test to keep pointers valid.
            let mut rand = alloc::vec::Vec::with_capacity(MAX_PAYLOAD_SIZE);
            let mut data = alloc::vec::Vec::new();

            for _ in 0..NUM_ITER {
                rand.clear();
                data.clear();

                let value = <$ty>::rand(&mut rand);

                $crate::ZEncode::z_encode(&value, &mut data).unwrap();

                let ret = <$ty as $crate::ZDecode>::z_decode(&mut &data[..]).unwrap();

                assert_eq!(ret, value);
            }
        }
    }};

    (ext, $ty:ty) => {{
        let mut rand = [0u8; MAX_PAYLOAD_SIZE];
        let mut data = [0u8; MAX_PAYLOAD_SIZE];

        for _ in 0..NUM_ITER {
            let value = <$ty>::rand(&mut &mut rand[..]);

            $crate::zext_encode::<_, 0x1, true>(&value, &mut &mut data[..], false).unwrap();

            let ret = $crate::zext_decode::<$ty>(&mut &data[..]).unwrap();

            assert_eq!(ret, value);
        }

        #[cfg(feature = "alloc")]
        {
            // Because random data generation uses the `ZStoreable` unsafe trait, we need
            // to avoid reallocation during the test to keep pointers valid.
            let mut rand = alloc::vec::Vec::with_capacity(MAX_PAYLOAD_SIZE);
            let mut data = alloc::vec::Vec::new();

            for _ in 0..NUM_ITER {
                rand.clear();
                data.clear();

                let value = <$ty>::rand(&mut rand);

                $crate::zext_encode::<_, 0x1, true>(&value, &mut data, false).unwrap();

                let ret = $crate::zext_decode::<$ty>(&mut &data[..]).unwrap();

                assert_eq!(ret, value);
            }
        }
    }};
}

macro_rules! roundtrips {
    (ext, $namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ext_ $ty:lower>]() {
                    roundtrip!(ext, $ty);
                }
            }
        )*
    };

    ($namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ $ty:lower>]() {
                    roundtrip!($ty);
                }
            }
        )*
    };
}

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

roundtrips!(ext, zenoh, EntityGlobalId, SourceInfo, Value, Attachment);
roundtrips!(zenoh, Err, Put, Query, Reply,);

roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    QueryableInfo
);

roundtrips!(
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

roundtrips!(ext, transport, Auth, Patch);
roundtrips!(
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

impl ZFramed for FrameBody<'_> {}

impl<'a> FrameBody<'a> {
    pub(crate) fn rand(w: &mut impl crate::ZStoreable<'a>) -> Self {
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

    pub fn is(&self, x: &Message<'a>) -> bool {
        match (self, x) {
            (FrameBody::Push(x), Message::Push { body: y, .. }) => x == y,
            (FrameBody::Request(x), Message::Request { body: y, .. }) => x == y,
            (FrameBody::Response(x), Message::Response { body: y, .. }) => x == y,
            (FrameBody::ResponseFinal(x), Message::ResponseFinal { body: y, .. }) => x == y,
            (FrameBody::Interest(x), Message::Interest { body: y, .. }) => x == y,
            (FrameBody::InterestFinal(x), Message::InterestFinal { body: y, .. }) => x == y,
            (FrameBody::Declare(x), Message::Declare { body: y, .. }) => x == y,
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
    let mut batch = BatchWriter::new(&mut data[..], 0);

    for msg in &messages {
        batch
            .framed(msg, Reliability::default(), QoS::default())
            .unwrap();
    }

    let (_, len) = batch.finalize();
    let batch = BatchReader::new(&data[..len]);

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
    let mut batch = BatchWriter::new(&mut data[..], 0);

    for (r, msg) in &messages {
        batch.framed(msg, *r, QoS::default()).unwrap();
    }

    batch.unframed(&KeepAlive {}).unwrap();

    let (_, len) = batch.finalize();
    let batch = BatchReader::new(&data[..len]);

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
            Message::KeepAlive(_) => {
                got_keepalive = true;
            }
            _ => panic!("First messages should be Frames, and last a KeepAlive"),
        }
    }

    assert!(messages.is_empty());
    assert!(got_keepalive);
}
