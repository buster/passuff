//! Frozen black-box runner. It executes the built `passuff` and `passuff-tui`
//! binaries and fails when a scenario, a step, or a parse produces no result.

mod harness;
mod steps;

use std::io::Write;

use cucumber::writer::{Ext as _, Stats as _};
use cucumber::World as _;

use harness::world::World;

#[tokio::main]
async fn main() {
    if let Err(message) = harness::bin::verify_binaries() {
        eprintln!("passuff-e2e: {message}");
        std::process::exit(2);
    }

    let features = format!("{}/features", env!("CARGO_MANIFEST_DIR"));

    let writer = World::cucumber()
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .with_writer(cucumber::writer::Basic::stdout().summarized())
        .run(&features)
        .await;

    let stats = writer.scenarios_stats();
    let parsing_errors = writer.parsing_errors();
    let total = stats.total() + parsing_errors;
    let failed = stats.failed + stats.skipped + parsing_errors;
    let passed = stats.passed;
    let skipped = stats.skipped;
    if let Ok(path) = std::env::var("GIVN_RESULT_FILE") {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = std::fs::File::create(path) {
            let _ = write!(
                file,
                "{{\"failed\":{failed},\"passed\":{passed},\"skipped\":{skipped},\"total\":{total}}}"
            );
        }
    }
    println!("givn result: total {total}, passed {passed}, failed {failed}, skipped {skipped}");
    if total == 0 {
        eprintln!("passuff-e2e: no scenario ran; the frozen package is misconfigured");
    }
    if failed > 0 || total == 0 {
        std::process::exit(1);
    }
}
