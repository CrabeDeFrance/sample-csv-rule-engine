use criterion::{Criterion, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("array_1_rule", |b| {
        b.iter(|| {
            let csv =
                csv_ruler::csv::read_from_str("A;B;C\n1;2;3\n1;4;5\nalpha;beta;alpha").unwrap();
            let rules = csv_ruler::rule::read_from_str("[{\"rule\": \"A == C\"}]").unwrap();

            let _ = csv_ruler::process::process_file(csv, &rules);
        });
    });

    c.bench_function("array_2_rules", |b| {
        b.iter(|| {
            let csv =
                csv_ruler::csv::read_from_str("A;B;C\n1;2;3\n1;4;5\nalpha;beta;alpha").unwrap();
            let rules = csv_ruler::rule::read_from_str(
                "[{\"rule\": \"A == C\"}, {\"rule\": \"(A + B) == C\"}]",
            )
            .unwrap();

            let _ = csv_ruler::process::process_file(csv, &rules);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
