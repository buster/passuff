use cucumber::{then, when};
use regex::Regex;

use crate::support::World;

#[when(regex = r#"^the developer lists the inbox$"#)]
async fn lists_inbox(world: &mut World) {
    if world.e2e {
        world.e2e_run_inbox(false).await;
    } else {
        world.run_inbox(false).await;
    }
}

#[when(regex = r#"^the developer lists all activity$"#)]
async fn lists_all(world: &mut World) {
    if world.e2e {
        world.e2e_run_inbox(true).await;
    } else {
        world.run_inbox(true).await;
    }
}

#[when(regex = r#"^the developer marks the newest activity read$"#)]
async fn marks_newest(world: &mut World) {
    let id = world.newest_activity_id().await;
    world.run_read(Some(id), false).await;
}

#[when(regex = r#"^the developer marks all activity read$"#)]
async fn marks_all(world: &mut World) {
    if world.e2e {
        world.e2e_run_read_all().await;
    } else {
        world.run_read(None, true).await;
    }
}

#[when(regex = r#"^the developer marks activity "([^"]+)" read$"#)]
async fn marks_id(world: &mut World, id: String) {
    world.run_read(Some(id), false).await;
}

#[when(regex = r#"^the developer reopens passuff$"#)]
async fn reopens(world: &mut World) {
    let state = world.state_json();
    assert!(
        state["activity"]
            .as_array()
            .is_some_and(|records| records.iter().any(|record| record["read"] == true)),
        "read state was not persisted before the restart: {state}"
    );
    world.last_report = None;
    world.last_transcript.clear();
}

#[when(regex = r#"^the developer runs read with no target$"#)]
async fn read_without_target(world: &mut World) {
    world.run_read(None, false).await;
}

#[when(regex = r#"^the developer runs the notification test$"#)]
async fn runs_notification_test(world: &mut World) {
    world.run_notify_test().await;
}

fn record_lines(transcript: &str) -> Vec<&str> {
    let pattern = Regex::new(r"^\s+a\d+\s+\d{4}-\d{2}-\d{2}T").expect("record regex");
    transcript
        .lines()
        .filter(|line| pattern.is_match(line))
        .collect()
}

#[then(regex = r#"^the inbox shows "([^"]+)" before "([^"]+)" for "([^"]+)"$"#)]
async fn shows_before(world: &mut World, first: String, second: String, key: String) {
    let lines: Vec<&str> = world.last_transcript.lines().collect();
    let first_index = lines
        .iter()
        .position(|line| line.contains(&key) && line.contains(&first))
        .unwrap_or_else(|| panic!("missing {first:?} for {key} in {:?}", world.last_transcript));
    let second_index = lines
        .iter()
        .position(|line| line.contains(&key) && line.contains(&second))
        .unwrap_or_else(|| {
            panic!(
                "missing {second:?} for {key} in {:?}",
                world.last_transcript
            )
        });
    assert!(
        first_index < second_index,
        "wrong order in {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^every inbox record is unread$"#)]
async fn every_record_unread(world: &mut World) {
    let lines = record_lines(&world.last_transcript);
    assert!(
        !lines.is_empty(),
        "no inbox records: {:?}",
        world.last_transcript
    );
    for line in lines {
        assert!(line.contains(" unread "), "record not unread: {line}");
    }
}

#[then(regex = r#"^the inbox shows an activity id and observation time$"#)]
async fn shows_id_and_time(world: &mut World) {
    let lines = record_lines(&world.last_transcript);
    assert!(
        !lines.is_empty(),
        "no inbox records: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the inbox shows "([^"]+)" for "([^"]+)" as (read|unread)$"#)]
async fn shows_state(world: &mut World, summary: String, key: String, state: String) {
    let needle = format!(" {state} ");
    assert!(
        world
            .last_transcript
            .lines()
            .any(|line| line.contains(&key) && line.contains(&summary) && line.contains(&needle)),
        "missing {summary:?} for {key} as {state}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^no state file was created$"#)]
async fn no_state_created(world: &mut World) {
    assert!(
        !world.state_path().exists(),
        "state file was created at {}",
        world.state_path().display()
    );
}
