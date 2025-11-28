use rand::{Rng, thread_rng};

use crate::{exts::*, msgs::*, *};

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
    DeclareBody,
    Declare,
    Interest,
    Push,
    Request,
    Response,
    ResponseFinal,
    PushBody,
    RequestBody,
    ResponseBody
);

super::roundtrips!(ext, transport, Auth, Patch);
super::roundtrips!(
    transport,
    Close,
    FrameHeader,
    FrameBody,
    InitSyn,
    InitAck,
    KeepAlive,
    OpenSyn,
    OpenAck
);

#[test]
fn network_stream() {
    extern crate std;
    use std::collections::VecDeque;

    let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut rand_writer = rand_data.as_mut_slice();

    let mut messages = {
        let mut msgs = VecDeque::new();
        for _ in 1..thread_rng().gen_range(1..16) {
            msgs.push_back(FrameBody::rand(&mut rand_writer));
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
    let frame = Frame {
        header: FrameHeader {
            reliability: Reliability::Reliable,
            sn: 0,
            qos: QoS::DEFAULT,
        },
        msgs: &mut reader,
    };

    for msg in frame {
        let actual = messages.pop_front().unwrap();
        assert_eq!(actual, msg.unwrap());
    }
}

#[test]
fn transport_stream() {
    extern crate std;
    use std::collections::VecDeque;

    let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut rand_writer = rand_data.as_mut_slice();

    let mut messages = {
        let mut msgs = VecDeque::new();
        for _ in 1..thread_rng().gen_range(1..16) {
            msgs.push_back((
                Reliability::rand(&mut rand_writer),
                FrameBody::rand(&mut rand_writer),
            ));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut batch = BatchWriter::new(&mut data, 0);

    for (r, msg) in &messages {
        batch.write_msg(msg, *r, QoS::DEFAULT).unwrap();
    }

    batch.write_keepalive().unwrap();

    let (_, len) = batch.finalize();
    let mut reader = &data[..len];
    let mut batch = BatchReader::new(&mut reader);

    let mut got_keepalive = false;
    while let Some(expected) = batch.next() {
        match expected.unwrap() {
            TransportBody::Frame(frame) => {
                let reliability = frame.header.reliability;
                for msg in frame {
                    let (r, actual) = messages.pop_front().unwrap();
                    assert_eq!(r, reliability);
                    assert_eq!(actual, msg.unwrap());
                }
            }
            TransportBody::KeepAlive(_) => {
                assert!(messages.is_empty());
                got_keepalive = true;
            }
            _ => panic!("First message should be a Frame, and last a KeepAlive"),
        }
    }
    assert!(got_keepalive);
}
