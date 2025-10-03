use criterion::{criterion_group, criterion_main, Criterion};
use zenoh_buffer::ZBufMut;
use zenoh_codec::{RCodec, WCodec, ZCodec};

fn criterion_benchmark(c: &mut Criterion) {
    let codec = ZCodec;

    let mut buff = [0u8; 16];
    let mut zbuf = ZBufMut(&mut buff);
    c.bench_function("u64 [0u8; 16]", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            codec.write(u64::MAX, &mut writer).unwrap();
            let mut reader = zbuf.reader();
            let _: u64 = codec.read(&mut reader).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
