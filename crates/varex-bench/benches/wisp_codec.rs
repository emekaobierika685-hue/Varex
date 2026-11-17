// crates/varex-bench/benches/wisp_codec.rs
use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, Criterion};
use varex_wisp::protocol::{Encoder, Packet};

fn bench_data(c: &mut Criterion) {
    let payload = vec![0u8; 32 * 1024];
    c.bench_function("encode_data_32k", |b| b.iter(|| {
        let mut buf = BytesMut::with_capacity(32 * 1024 + 5);
        Encoder::data(&mut buf, 7, &payload);
    }));
    let mut buf = BytesMut::new();
    Encoder::data(&mut buf, 7, &payload);
    let frozen = buf.freeze();
    c.bench_function("parse_data_32k", |b| b.iter(|| Packet::parse(frozen.clone()).unwrap()));
}

criterion_group!(benches, bench_data);
criterion_main!(benches);
