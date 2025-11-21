use criterion::Criterion;

use crate::{
    Encoding, Reliability, WireExpr, ZDecode, ZEncode, ZLen, keyexpr,
    network::{NodeId, QoS, push::Push},
    transport::{TransportBatch, frame::FrameHeader},
    zenoh::{PushBody, put::Put},
};

#[test]
#[ignore]
fn bench_codec() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = [0u8; 9];
    c.bench_function("encode_u64", |b| {
        b.iter(|| {
            let mut w = data.as_mut_slice();
            <_ as ZEncode>::z_encode(&u64::MAX, &mut w).unwrap();
            let mut r = data.as_slice();
            let _ = <u64 as ZDecode>::z_decode(&mut r).unwrap();
        })
    });

    let mut data = [0u8; 16];
    c.bench_function("encode_hello_world", |b| {
        b.iter(|| {
            let mut w = data.as_mut_slice();
            let len = <_ as ZLen>::z_len(&"Hello, World!");
            <_ as ZEncode>::z_encode(&"Hello, World!", &mut w).unwrap();
            let mut r = &data[..len];
            let _ = <&str as ZDecode>::z_decode(&mut r).unwrap();
        })
    });

    let mut data = [0u8; u16::MAX as usize];
    let frame = FrameHeader {
        reliability: Reliability::DEFAULT,
        sn: 0,
        qos: QoS::DEFAULT,
    };

    let msg = Push {
        wire_expr: WireExpr::from(keyexpr::new("demo/example").unwrap()),
        qos: QoS::DEFAULT,
        timestamp: None,
        nodeid: NodeId::DEFAULT,
        payload: PushBody::Put(Put {
            timestamp: None,
            encoding: Encoding::DEFAULT,
            sinfo: None,
            attachment: None,
            payload: &[0u8; 8],
        }),
    };

    c.bench_function("encode_batch", |b| {
        b.iter(|| {
            let mut w = data.as_mut_slice();
            <_ as ZEncode>::z_encode(&frame, &mut w).unwrap();
            while <_ as ZEncode>::z_encode(&msg, &mut w).is_ok() {}
        })
    });

    let mut w = data.as_mut_slice();
    <_ as ZEncode>::z_encode(&frame, &mut w).unwrap();
    while <_ as ZEncode>::z_encode(&msg, &mut w).is_ok() {}
    let len = u16::MAX as usize - w.len();
    c.bench_function("decode_batch", |b| {
        b.iter(|| {
            let mut r = &data[..len];
            let mut iter = TransportBatch::new(&mut r);
            while iter.next().is_some() {}
        })
    });

    Criterion::default().final_summary();
}
