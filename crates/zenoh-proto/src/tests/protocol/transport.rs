use core::panic;

use rand::{Rng, thread_rng};

use crate::{
    Reliability,
    network::{NetworkBody, QoS},
    transport::{close::*, frame::*, init::*, keepalive::*, open::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

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
                NetworkBody::rand(&mut rand_writer),
            ));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut batch = Batch::new(&mut data, 0);

    for (r, msg) in &messages {
        batch.write_msg(msg, *r, QoS::DEFAULT).unwrap();
    }

    batch.write_keepalive().unwrap();

    let (_, len) = batch.finalize();
    let mut reader = &data[..len];
    let mut iter = TransportBatch::new(&mut reader);

    let mut got_keepalive = false;
    while let Some(expected) = iter.next().unwrap() {
        match expected {
            TransportBody::Frame(mut frame) => {
                for msg in frame.msgs.by_ref() {
                    let (r, actual) = messages.pop_front().unwrap();
                    assert_eq!(r, frame.header.reliability);
                    assert_eq!(actual, msg);
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
