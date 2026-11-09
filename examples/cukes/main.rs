use std::path::PathBuf;

use mizeria::prelude::*;

#[derive(Default, Clone)]
struct Cukes(u32);

fn main() {
    let features_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/cukes/features");

    let features = Features::builder()
        .add_path(features_dir)
        .build()
        .expect("failed to load features");

    Suite::new(|| ())
        .given(
            re(r"I have (\d+) cukes"),
            |n: u32, cukes: GetOrDefault<Cukes>| Put::new(Cukes(cukes.0 + n)),
        )
        .with_reporter(ColourSyncPrinter::stdout().symbols(true))
        .run_blocking(&features)
        .exit()
}
