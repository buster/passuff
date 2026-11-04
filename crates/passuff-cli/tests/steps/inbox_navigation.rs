use std::time::Duration;

use cucumber::{given, then, when};
use passuff_core::Comment;

use crate::support::World;

#[when(regex = r#"^the developer opens the inbox TUI with keys "([^"]+)"$"#)]
async fn opens_with_keys(world: &mut World, keys: String) {
    if world.e2e {
        world.e2e_run_tui_snapshot_keys(&keys).await;
    } else {
        world.run_tui_snapshot().await;
        for key in keys.split(',').map(str::trim).filter(|key| !key.is_empty()) {
            world.run_tui_press(key).await;
        }
    }
}

#[when(regex = r#"^the developer moves the selection down (\d+) times$"#)]
async fn moves_down(world: &mut World, count: usize) {
    world.press_many("j", count).await;
}

#[then(regex = r#"^the focused pane is "(PRs|activity)"$"#)]
async fn focused(world: &mut World, expected: String) {
    world.assert_focus(&expected);
}

#[then(regex = r#"^the selected pull request is "([^"]+)"$"#)]
async fn selected_pr(world: &mut World, key: String) {
    world.assert_selected_pull_request(&key);
}

#[given(regex = r#"^(\d+) authored pull requests in "([^"]+)" by "([^"]+)"$"#)]
async fn many_pull_requests(world: &mut World, count: u64, repository: String, author: String) {
    world.add_many_pull_requests(&repository, &author, count);
}

#[given(regex = r#"^the watcher has already observed all of them$"#)]
async fn observed_all(world: &mut World) {
    world.pending_baseline = true;
    world.ensure_baseline().await;
}

#[when(regex = r#"^an authored pull request "([^"]+)" by "([^"]+)" appears$"#)]
async fn appears(world: &mut World, key: String, author: String) {
    world.add_pr(&key, &author);
    world.pending_baseline = true;
    world.ensure_baseline().await;
}

#[when(regex = r#"^the developer refreshes the inbox$"#)]
async fn refreshes(world: &mut World) {
    world.refresh_tui().await;
}

#[when(regex = r#"^the developer opens the inbox TUI and lets it observe for "(\d+)" seconds$"#)]
async fn opens_observing(world: &mut World, seconds: u64) {
    if world.e2e {
        world.e2e_run_tui_observe_for(seconds * 1000).await;
    } else {
        world.run_tui_snapshot().await;
    }
}

#[given(regex = r#"^"([^"]+)" comments "(.*)" on "([^"]+)" after the first observation$"#)]
async fn delayed_comment(world: &mut World, author: String, body: String, key: String) {
    world.e2e_pending_comment = Some((
        key,
        "comment-delayed".to_string(),
        author,
        body,
    ));
}

#[when(regex = r#"^the pull request "([^"]+)" receives a long comment from "([^"]+)"$"#)]
async fn long_comment(world: &mut World, key: String, author: String) {
    world.ensure_baseline().await;
    let body = (1..=40)
        .map(|line| format!("comment line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let comment = Comment {
        id: "long-comment".to_string(),
        author,
        body,
        created_at: "2026-11-02T12:00:01Z".to_string(),
    };
    world.pr_mut(&key).comments.push(comment.clone());
    world.pr_mut(&key).updated_at = comment.created_at.clone();
    if world.e2e {
        world
            .e2e_set_comment(&key, &comment.id, &comment.author, &comment.body)
            .await;
    }
}

#[when(regex = r#"^the inbox observes the account three times with no change$"#)]
async fn observes_three(world: &mut World) {
    for _ in 0..3 {
        world.observe_account().await;
    }
}

#[when(regex = r#"^the inbox observes the account$"#)]
async fn observes(world: &mut World) {
    world.observe_account().await;
}

#[then(regex = r#"^the inbox has asked GitHub for the full account history "(\d+)" times?$"#)]
async fn full_history(world: &mut World, count: usize) {
    assert_eq!(
        world.observation_observe_calls(),
        count,
        "full observation count differs"
    );
}

#[then(regex = r#"^the inbox has asked GitHub for a change signal "(\d+)" times$"#)]
async fn change_signals(world: &mut World, count: usize) {
    assert_eq!(
        world.observation_probe_calls(),
        count,
        "change probe count differs"
    );
}

#[given(regex = r#"^the account has "(\d+)" API points left until the allowance resets in "(\d+)" seconds$"#)]
async fn allowance(world: &mut World, remaining: u32, reset: i64) {
    world.set_observation_allowance(remaining, reset);
}

#[when(regex = r#"^the inbox plans its next observation$"#)]
async fn plans(world: &mut World) {
    world.plan_observation_delay();
}

#[then(regex = r#"^the inbox waits at least "(\d+)" seconds before observing again$"#)]
async fn waits(world: &mut World, seconds: u64) {
    let delay = world.planned_delay().expect("planned delay");
    assert!(
        delay >= Duration::from_secs(seconds),
        "planned delay {delay:?} is shorter than {seconds}s"
    );
}

#[when(regex = r#"^GitHub becomes unavailable for the next observation$"#)]
async fn unavailable(world: &mut World) {
    world.fail_next_observation();
}

#[then(regex = r#"^the TUI still shows "([^"]+)"$"#)]
async fn still_shows(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected TUI to still contain {expected:?}: {:?}",
        world.last_transcript
    );
}

#[given(regex = r#"^the reconciliation interval is "(\d+)" seconds$"#)]
async fn reconciliation_interval(world: &mut World, seconds: u64) {
    world.reconciliation_interval_secs = Some(seconds);
}

#[when(regex = r#"^the inbox waits "(\d+)" seconds$"#)]
async fn inbox_waits(_world: &mut World, seconds: u64) {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
}

#[then(regex = r#"^the inbox has observed only "([^"]+)"$"#)]
async fn observed_only(world: &mut World, key: String) {
    assert_eq!(
        world.observation_observed_targets(),
        vec![format!("node-{key}")],
        "observation target set differs"
    );
}
