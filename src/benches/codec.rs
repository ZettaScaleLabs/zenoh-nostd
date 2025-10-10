use std::u64;

use criterion::{Criterion, criterion_group, criterion_main};
use zenoh_nostd::{
    protocol::{
        core::{Reliability, encoding::Encoding, wire_expr::WireExpr},
        network::{self, NetworkBody, NetworkMessage, push::Push},
        transport::{
            self, TransportMessage, TransportSn,
            frame::FrameHeader,
            init::{InitAck, InitSyn},
            open::{OpenAck, OpenSyn},
        },
        zcodec::{decode_str, decode_u64, encode_str, encode_u64},
        zenoh::{PushBody, put::Put},
    },
    result::ZResult,
    zbuf::{ZBufMut, ZBufMutExt},
};

fn criterion_benchmark(c: &mut Criterion) {
    let mut buff = [0u8; 16];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode u64", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            encode_u64(u64::MAX, &mut writer).unwrap();
            let mut reader = zbuf.reader();
            let _: u64 = decode_u64(&mut reader).unwrap();
        })
    });

    let mut buff = [0u8; 64];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode b'Hello, world!'", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            encode_str(true, "Hello, world!", &mut writer).unwrap();
            let mut reader = zbuf.reader();
            let _: &str = decode_str(None, &mut reader).unwrap();
        })
    });

    let mut buff = [0u8; u16::MAX as usize];

    let frame = FrameHeader {
        reliability: Reliability::DEFAULT,
        sn: TransportSn::MIN,
        ext_qos: transport::frame::ext::QoSType::DEFAULT,
    };

    let msg = NetworkMessage {
        reliability: Reliability::BestEffort,
        body: NetworkBody::Push(Push {
            wire_expr: WireExpr::from("demo/example"),
            ext_qos: network::push::ext::QoSType::DEFAULT,
            ext_nodeid: network::push::ext::NodeIdType::DEFAULT,
            ext_tstamp: None,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::empty(),
                ext_sinfo: None,
                ext_attachment: None,
                payload: &[0u8; 8],
            }),
        }),
    };

    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode Batch NetworkMsg", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            frame.encode(&mut writer).unwrap();
            while msg.encode(&mut writer).is_ok() {}
        })
    });

    let mut buff = [0u8; u16::MAX as usize];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    let mut writer = zbuf.writer();
    frame.encode(&mut writer).unwrap();
    while msg.encode(&mut writer).is_ok() {}
    c.bench_function("Decode Batch NetworkMsg", |b| {
        b.iter(|| {
            let mut reader = zbuf.reader();
            TransportMessage::decode_batch(
                &mut reader,
                None::<fn(InitSyn) -> ZResult<()>>,
                None::<fn(InitAck) -> ZResult<()>>,
                None::<fn(OpenSyn) -> ZResult<()>>,
                None::<fn(OpenAck) -> ZResult<()>>,
                None::<fn() -> ZResult<()>>,
                None::<fn(&FrameHeader, NetworkMessage) -> ZResult<()>>,
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
