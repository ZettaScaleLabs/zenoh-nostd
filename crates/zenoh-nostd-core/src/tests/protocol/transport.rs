use rand::{Rng, thread_rng};

use crate::{
    ZEncode,
    network::NetworkBody,
    transport::{close::*, frame::*, init::*, keepalive::*, open::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, transport, QoSLink, Auth, MultiLink, PatchType);
crate::roundtrips!(
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
            msgs.push_back(NetworkBody::rand(&mut rand_writer));
        }
        msgs
    };

    let mut data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut writer = data.as_mut_slice();
    let start = writer.len();
    <_ as ZEncode>::z_encode(&FrameHeader::rand(&mut rand_writer), &mut writer).unwrap();

    for msg in &messages {
        <_ as ZEncode>::z_encode(msg, &mut writer).unwrap();
    }
    let len = start - writer.len();

    let mut reader = &data[..len];
    let mut iter = TransportBodyIter::new(&mut reader);

    while let Some(expected) = iter.next() {
        match expected {
            TransportBody::Frame(mut frame) => {
                while let Some(msg) = frame.iter.next() {
                    let actual = messages.pop_front().unwrap();
                    assert_eq!(actual, msg);
                }
            }
            _ => panic!("First message should be a Frame"),
        }
    }
}
