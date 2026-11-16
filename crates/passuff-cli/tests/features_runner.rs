#![allow(dead_code, unused_imports)]
mod steps;
mod support;

use std::io::Write;

use cucumber::writer::{Ext as _, Stats as _};
use cucumber::World as _;
use support::world::World;

#[tokio::main]
async fn main() {
    let features = std::env::var("GIVN_FEATURES").unwrap_or_else(|_| "givn/specs".to_string());
    let scope = std::env::var("GIVN_RESULT_SCOPE").unwrap_or_else(|_| "regular".to_string());
    let e2e_only = scope == "e2e";
    let filter = move |_feature: &cucumber::gherkin::Feature,
                       _rule: Option<&cucumber::gherkin::Rule>,
                       scenario: &cucumber::gherkin::Scenario| {
        let is_e2e = scenario
            .tags
            .iter()
            .any(|tag| tag.trim_start_matches('@') == "e2e");
        if e2e_only {
            is_e2e
        } else if scope == "change" || scope == "regular" {
            e2e_only || !is_e2e
        } else {
            !is_e2e
        }
    };

    let writer = World::cucumber()
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .with_writer(cucumber::writer::Basic::stdout().summarized())
        .filter_run(&features, filter)
        .await;

    let stats = writer.scenarios_stats();
    let parsing_errors = writer.parsing_errors();
    let total = stats.total() + parsing_errors;
    let failed = stats.failed + parsing_errors + stats.skipped;
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
    if failed > 0 {
        std::process::exit(1);
    }
}
