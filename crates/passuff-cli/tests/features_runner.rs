mod support;

#[path = "steps/activity_inbox.rs"]
mod activity_inbox_steps;
#[path = "steps/activity_inbox_tui.rs"]
mod activity_inbox_tui_steps;
#[path = "steps/pr_activity_notification_e2e.rs"]
mod e2e_steps;
#[path = "steps/pr_activity_notification.rs"]
mod regular_steps;
#[path = "steps/stacked_inbox.rs"]
mod stacked_inbox_steps;
#[path = "steps/watch_scope.rs"]
mod watch_scope_steps;
#[path = "steps/setup.rs"]
mod setup_steps;
#[path = "steps/watch_status.rs"]
mod watch_status_steps;

use std::path::PathBuf;

use cucumber::World as _;
use futures::FutureExt as _;
use serde_json::Value;
use support::World;

#[derive(Debug, Default)]
struct Counts {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
}

fn main() {
    let features = resolve_features(
        &std::env::var("GIVN_FEATURES").unwrap_or_else(|_| "givn/specs".to_string()),
    );
    let scope = std::env::var("GIVN_RESULT_SCOPE").unwrap_or_else(|_| "change".to_string());
    let result_file = std::env::var("GIVN_RESULT_FILE").ok();
    let scenario_filter = std::env::var("GIVN_SCENARIO").ok();
    if let Some(path) = result_file.as_deref() {
        let _ = std::fs::remove_file(path);
    }
    let output = tempfile::NamedTempFile::new().expect("json output");
    let output_path = output.path().to_path_buf();
    let json_writer = cucumber::writer::Json::new::<World>(output.reopen().expect("json file"));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    runtime.block_on(async move {
        let _writer = World::cucumber()
            .before(|_, _, scenario, world| {
                world.e2e = scenario.tags.iter().any(|tag| tag == "e2e");
                world.e2e_scenario = slugify(&scenario.name);
                async {}.boxed_local()
            })
            .with_writer(json_writer)
            .fail_on_skipped()
            .max_concurrent_scenarios(Some(1))
            .filter_run(features, move |_, _, scenario| {
                let is_e2e = scenario.tags.iter().any(|tag| tag == "e2e");
                let matches_scope = match scope.as_str() {
                    "e2e" => is_e2e,
                    "regular" => !is_e2e,
                    _ => true,
                };
                let matches_name = scenario_filter
                    .as_deref()
                    .is_none_or(|name| scenario.name == name);
                matches_scope && matches_name
            })
            .await;
    });

    let text = std::fs::read_to_string(&output_path).unwrap_or_default();
    if let Ok(path) = std::env::var("PASSUFF_CUCUMBER_JSON") {
        let _ = std::fs::write(path, &text);
    }
    print_failures(&text);
    let counts = count_scenarios(&text);
    let payload = serde_json::json!({
        "total": counts.total,
        "passed": counts.passed,
        "failed": counts.failed,
        "skipped": counts.skipped,
    });
    if let Some(path) = result_file {
        let path = PathBuf::from(path);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, format!("{payload}\n"));
    }
    println!(
        "givn test-result: total {}, passed {}, failed {}, skipped {}",
        counts.total, counts.passed, counts.failed, counts.skipped
    );
    if counts.total == 0 || counts.failed > 0 || counts.skipped > 0 {
        std::process::exit(1);
    }
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn resolve_features(features: &str) -> String {
    let path = std::path::Path::new(features);
    if path.is_absolute() || path.exists() {
        return features.to_string();
    }
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidate = workspace.join(path);
    if candidate.exists() {
        candidate.to_string_lossy().into_owned()
    } else {
        features.to_string()
    }
}

fn print_failures(text: &str) {
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Array(Vec::new()));
    let Some(features) = value.as_array() else {
        return;
    };
    for feature in features {
        let feature_name = feature.get("name").and_then(Value::as_str).unwrap_or("?");
        let Some(elements) = feature.get("elements").and_then(Value::as_array) else {
            continue;
        };
        for element in elements {
            if element.get("type").and_then(Value::as_str) != Some("scenario") {
                continue;
            }
            let mut failed = false;
            let mut detail = String::new();
            if let Some(steps) = element.get("steps").and_then(Value::as_array) {
                for step in steps {
                    let status = step
                        .get("result")
                        .and_then(|r| r.get("status"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if matches!(
                        status,
                        "failed" | "undefined" | "ambiguous" | "pending" | "skipped"
                    ) {
                        failed = true;
                        detail.push_str(&format!(
                            "\n  {} {} -> {}",
                            status,
                            step.get("keyword").and_then(Value::as_str).unwrap_or(""),
                            step.get("name").and_then(Value::as_str).unwrap_or("")
                        ));
                        if let Some(err) = step
                            .get("result")
                            .and_then(|r| r.get("error_message"))
                            .and_then(Value::as_str)
                        {
                            detail.push_str(&format!("\n    {err}"));
                        }
                    }
                }
            }
            if failed {
                eprintln!(
                    "FAILED: {} :: {}",
                    feature_name,
                    element.get("name").and_then(Value::as_str).unwrap_or("?")
                );
                eprintln!("{detail}");
            }
        }
    }
}

fn count_scenarios(text: &str) -> Counts {
    let mut counts = Counts::default();
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Array(Vec::new()));
    let Some(features) = value.as_array() else {
        return counts;
    };
    for feature in features {
        let Some(elements) = feature.get("elements").and_then(Value::as_array) else {
            continue;
        };
        for element in elements {
            if element.get("type").and_then(Value::as_str) != Some("scenario") {
                continue;
            }
            counts.total += 1;
            let mut status = "passed";
            if let Some(steps) = element.get("steps").and_then(Value::as_array) {
                for step in steps {
                    match step
                        .get("result")
                        .and_then(|result| result.get("status"))
                        .and_then(Value::as_str)
                    {
                        Some("failed" | "undefined" | "ambiguous" | "pending") => {
                            status = "failed";
                            break;
                        }
                        Some("skipped") => status = "skipped",
                        _ => {}
                    }
                }
            }
            match status {
                "passed" => counts.passed += 1,
                "failed" => counts.failed += 1,
                "skipped" => counts.skipped += 1,
                _ => {}
            }
        }
    }
    counts
}
