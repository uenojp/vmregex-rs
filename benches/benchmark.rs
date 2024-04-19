use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use vmregex::Regex;

pub fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("a?^na^n benchmark");
    group.measurement_time(Duration::from_secs(1));

    let inputs = (1..=8).map(|n| (n, ("a?".repeat(n) + &"a".repeat(n), "a".repeat(n))));

    for (n, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new(format!("n={n}"), 0),
            &input,
            |b, (pattern, text)| {
                b.iter(|| {
                    let re = Regex::new(&pattern).unwrap();
                    re.is_match(&text).unwrap();
                })
            },
        );
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
