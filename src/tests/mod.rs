use criterion::Criterion;

mod codec;

fn criterion_benchmark(c: &mut Criterion) {
    codec::criterion(c);
}

#[test]
fn bench() {
    extern crate std;
    use std::{
        string::{String, ToString},
        vec::Vec,
    };

    let args: Vec<String> = std::env::args().collect();

    let filter = args
        .windows(3)
        .filter(|p| p.len() >= 2 && p[0].ends_with("bench") && p[1] == "--")
        .map(|s| s.get(2).unwrap_or(&"".to_string()).clone())
        .next();

    let mut c = Criterion::default().with_output_color(true).without_plots();

    if let Some(f) = filter {
        c = c.with_filter(f);
    }

    criterion_benchmark(&mut c);

    Criterion::default().final_summary();
}
