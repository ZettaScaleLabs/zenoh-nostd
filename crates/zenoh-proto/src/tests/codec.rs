use criterion::Criterion;

use crate::{fields::*, keyexpr, msgs::*, *};

#[test]
#[ignore]
fn bench_codec_u64_array() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = [0u8; 9];
    c.bench_function("encode_u64_array", |b| {
        b.iter(|| {
            crate::ZEncode::z_encode(&u64::MAX, &mut &mut data[..]).unwrap();
            let _ = <u64 as crate::ZDecode>::z_decode(&mut &data[..]).unwrap();
        })
    });

    c.final_summary();
}

#[cfg(feature = "alloc")]
#[test]
#[ignore]
fn bench_codec_u64_vec() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = alloc::vec::Vec::new();
    c.bench_function("encode_u64_vec", |b| {
        b.iter(|| {
            data.clear();
            crate::ZEncode::z_encode(&u64::MAX, &mut data).unwrap();
            let _ = <u64 as crate::ZDecode>::z_decode(&mut &data[..]).unwrap();
        })
    });

    c.final_summary();
}

#[test]
#[ignore]
fn bench_codec_str_array() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = [0u8; 16];
    c.bench_function("encode_str_array", |b| {
        b.iter(|| {
            let len = crate::ZLen::z_len(&"Hello, World!");
            crate::ZEncode::z_encode(&"Hello, World!", &mut &mut data[..]).unwrap();
            let _ = <&str as crate::ZDecode>::z_decode(&mut &data[..len]).unwrap();
        })
    });

    c.final_summary();
}

#[cfg(feature = "alloc")]
#[test]
#[ignore]
fn bench_codec_str_vec() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = alloc::vec::Vec::new();
    c.bench_function("encode_str_vec", |b| {
        b.iter(|| {
            data.clear();
            let len = crate::ZLen::z_len(&"Hello, World!");
            crate::ZEncode::z_encode(&"Hello, World!", &mut data).unwrap();
            let _ = <&str as crate::ZDecode>::z_decode(&mut &data[..len]).unwrap();
        })
    });

    c.final_summary();
}

#[test]
#[ignore]
fn bench_codec_encode_batch_array() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = [0u8; u16::MAX as usize];
    let frame = FrameHeader::default();

    let msg = Push {
        wire_expr: WireExpr::from(keyexpr::new("demo/example").unwrap()),
        payload: PushBody::Put(Put {
            payload: &[0u8; 8],
            ..Default::default()
        }),
        ..Default::default()
    };

    c.bench_function("encode_batch_array", |b| {
        b.iter(|| {
            let mut w = data.as_mut_slice();

            crate::ZEncode::z_encode(&frame, &mut w).unwrap();
            while crate::ZEncode::z_encode(&msg, &mut w).is_ok() {}
        })
    });

    c.final_summary();
}

#[cfg(feature = "alloc")]
#[test]
#[ignore]
fn bench_codec_encode_batch_vec() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = alloc::vec::Vec::with_capacity(u16::MAX as usize);
    let frame = FrameHeader::default();

    let msg = Push {
        wire_expr: WireExpr::from(keyexpr::new("demo/example").unwrap()),
        payload: PushBody::Put(Put {
            payload: &[0u8; 8],
            ..Default::default()
        }),
        ..Default::default()
    };

    c.bench_function("encode_batch_vec", |b| {
        b.iter(|| {
            data.clear();

            crate::ZEncode::z_encode(&frame, &mut data).unwrap();
            while crate::ZEncode::z_encode(&msg, &mut data).is_ok() {
                if data.len() >= u16::MAX as usize {
                    break;
                }
            }
        })
    });

    c.final_summary();
}

#[test]
#[ignore]
fn bench_codec_decode_batch_array() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = [0u8; u16::MAX as usize];
    let frame = FrameHeader::default();

    let msg = Push {
        wire_expr: WireExpr::from(keyexpr::new("demo/example").unwrap()),
        payload: PushBody::Put(Put {
            payload: &[0u8; 8],
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut w = data.as_mut_slice();
    crate::ZEncode::z_encode(&frame, &mut w).unwrap();
    while crate::ZEncode::z_encode(&msg, &mut w).is_ok() {}

    let len = u16::MAX as usize - w.len();
    c.bench_function("decode_batch_array", |b| {
        b.iter(|| {
            let mut batch = BatchReader::new(&data[..len]);
            while batch.next().is_some() {}
        })
    });

    c.final_summary();
}

#[cfg(feature = "alloc")]
#[test]
#[ignore]
fn bench_codec_decode_batch_vec() {
    let mut c = Criterion::default().with_output_color(true);

    let mut data = alloc::vec::Vec::with_capacity(u16::MAX as usize);
    let frame = FrameHeader::default();

    let msg = Push {
        wire_expr: WireExpr::from(keyexpr::new("demo/example").unwrap()),
        payload: PushBody::Put(Put {
            payload: &[0u8; 8],
            ..Default::default()
        }),
        ..Default::default()
    };

    crate::ZEncode::z_encode(&frame, &mut data).unwrap();
    while crate::ZEncode::z_encode(&msg, &mut data).is_ok() {
        if data.len() >= u16::MAX as usize {
            break;
        }
    }

    c.bench_function("decode_batch_array", |b| {
        b.iter(|| {
            let mut batch = BatchReader::new(&data[..]);
            while batch.next().is_some() {}
        })
    });

    c.final_summary();
}
