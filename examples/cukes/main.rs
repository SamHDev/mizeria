use std::path::PathBuf;

use mizeria::cli::Harness;
use mizeria::prelude::*;

#[derive(Default, Clone)]
struct Cukes(u32);

fn main() {
    let features_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/cukes/features");

    let mut suite = Suite::new(|| ());
    suite
        .given(re(r"I have (\d+) cukes"), |n: u32| Put::new(Cukes(n)))
        .when(re(r"I eat (\d+) cukes"), |n: u32, cukes: Get<Cukes>| {
            Put::new(Cukes(cukes.0 - n))
        })
        .then(
            re(r"I have (\d+) cukes left"),
            |n: u32, cukes: GetRef<Cukes>| {
                if cukes.0 != n {
                    fail!("expected {n} cukes left, got {}", cukes.0);
                }
                Ok(())
            },
        );

    Harness::builder()
        .name("cukes")
        .about("Run the cukes example scenarios")
        .version(env!("CARGO_PKG_VERSION"))
        .add_path(features_dir)
        .run_sync_and_exit(&mut suite);
}
