use cucumber::{given, then, when};

use crate::support::{ci_pending, World};

#[given(regex = r#"^the watch scope is limited to "([^"]+)"$"#)]
async fn watch_scope(world: &mut World, repository: String) {
    world.watch_scope.push(repository);
}

#[given(regex = r#"^no state file$"#)]
async fn no_state(world: &mut World) {
    let _ = std::fs::remove_file(world.state_path());
}

#[given(regex = r#"^a state already watches "([^"]+)"$"#)]
async fn seeded_watched(world: &mut World, key: String) {
    world.seed_watched_pull_request(&key);
}

#[given(regex = r#"^a state with one activity record$"#)]
async fn seeded_one_record(world: &mut World) {
    world.seed_activity_records(1);
}

#[given(regex = r#"^a state with 500 activity records$"#)]
async fn seeded_records(world: &mut World) {
    world.seed_activity_records(500);
}

#[given(regex = r#"^a version-1 state file with one watched pull request$"#)]
async fn seeded_v1(world: &mut World) {
    world.seed_version_one_state();
}

#[given(regex = r#"^the CI result on "([^"]+)" is "([^"]+)"$"#)]
async fn ci_result_is(world: &mut World, key: String, state: String) {
    world.pr_mut(&key).ci = match state.as_str() {
        "pending" => ci_pending(),
        other => panic!("unsupported CI state {other}"),
    };
}

#[given(regex = r#"^the state directory cannot be written$"#)]
async fn state_readonly(world: &mut World) {
    world.fail_state_write = true;
}

#[when(regex = r#"^the developer asks for status$"#)]
async fn asks_for_status(world: &mut World) {
    if world.e2e {
        world.e2e_run_status().await;
    } else {
        world.run_status().await;
    }
}

#[then(regex = r#"^the transcript does not report "([^"]+)"$"#)]
async fn transcript_does_not_report(world: &mut World, unexpected: String) {
    assert!(
        !world.last_transcript.contains(&unexpected),
        "unexpected transcript text {unexpected:?}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the state file version is (\d+)$"#)]
async fn state_version(world: &mut World, version: u32) {
    let state = world.state_json();
    assert_eq!(state["version"], version, "{state}");
}

#[then(regex = r#"^the state retains (\d+) activity records$"#)]
async fn state_retains(world: &mut World, count: usize) {
    let state = world.state_json();
    assert_eq!(
        state["activity"].as_array().map(Vec::len).unwrap_or(0),
        count,
        "{state}"
    );
}

#[then(regex = r#"^the oldest activity record is gone$"#)]
async fn oldest_gone(world: &mut World) {
    let state = world.state_json();
    let ids: Vec<&str> = state["activity"]
        .as_array()
        .map(|records| {
            records
                .iter()
                .filter_map(|record| record["id"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert!(!ids.contains(&"a1"), "oldest record still present: {ids:?}");
}
