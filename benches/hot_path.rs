//! Hot-path benches: step matching and outline placeholder expansion.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use cutecumber::statement::{Regex, Statement};
use cutecumber::{Features, Suite};
use std::time::Duration;

fn many_regex_definitions(c: &mut Criterion) {
    let patterns: Vec<Regex> = (0..64)
        .map(|i| Regex::new(&format!(r"I have ({i}) cukes and (\d+) more")).unwrap())
        .collect();
    // Winning pattern is last so a naive scan pays full cost.
    let target = "I have 63 cukes and 9 more";

    c.bench_function("match_64_regex_last_wins", |b| {
        b.iter(|| {
            let mut found = None;
            for stmt in &patterns {
                if let Some(caps) = stmt.match_bag(black_box(target)) {
                    found = Some(caps);
                    break;
                }
            }
            black_box(found)
        })
    });
}

fn suite_keyword_filtered_match(c: &mut Criterion) {
    let dir = std::env::temp_dir().join("cutecumber-bench-kw");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("b.feature");
    std::fs::write(
        &path,
        "\
Feature: Bench
  Scenario: s
    When when step number 31 with 7
",
    )
    .unwrap();
    let features = Features::builder().add_path(&path).build().unwrap();

    c.bench_function("run_when_among_64_defs", |b| {
        b.iter_with_setup(
            || {
                let mut suite = Suite::new(|| 0u32);
                for i in 0..32 {
                    let pat = format!(r"given step number {i} with (\d+)");
                    suite.given(Regex::new(&pat).unwrap(), |_n: u32| {});
                }
                for i in 0..32 {
                    let pat = format!(r"when step number {i} with (\d+)");
                    suite.when(Regex::new(&pat).unwrap(), |_n: u32| {});
                }
                suite
            },
            |mut suite| {
                let outcome = suite.run_blocking(black_box(&features));
                black_box(outcome)
            },
        )
    });

    let _ = std::fs::remove_dir_all(&dir);
}

fn outline_substitute(c: &mut Criterion) {
    let values = vec![
        ("<name>".to_owned(), "Alice".to_owned()),
        ("<qty>".to_owned(), "42".to_owned()),
        ("<note>".to_owned(), "hello world".to_owned()),
    ];
    let text = "user <name> buys <qty> of <note> today";

    c.bench_function("outline_substitute_3_keys", |b| {
        b.iter(|| {
            let mut out = black_box(text).to_owned();
            for (needle, value) in &values {
                if out.contains(needle.as_str()) {
                    out = out.replace(needle.as_str(), value);
                }
            }
            black_box(out)
        })
    });
}

fn configure() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(2))
}

criterion_group! {
    name = benches;
    config = configure();
    targets = many_regex_definitions, suite_keyword_filtered_match, outline_substitute
}
criterion_main!(benches);
