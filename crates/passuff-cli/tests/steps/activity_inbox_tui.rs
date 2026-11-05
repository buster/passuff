use cucumber::{given, then, when};

use crate::support::World;

#[when(regex = r#"^the developer opens the inbox TUI$"#)]
async fn opens(world: &mut World) {
    world.run_tui_snapshot().await;
}

#[when(regex = r#"^the developer opens the inbox TUI observing the account$"#)]
async fn opens_observing(world: &mut World) {
    world.e2e_run_tui_observe().await;
}

#[when(regex = r#"^the developer starts the inbox TUI binary$"#)]
async fn starts_binary(world: &mut World) {
    world.run_tui_binary();
}

#[when(regex = r#"^the developer starts the inbox TUI without a snapshot$"#)]
async fn starts_binary_without_snapshot(world: &mut World) {
    world.run_tui_binary_without_snapshot();
}

#[given(regex = r#"^the state path is a directory$"#)]
async fn state_path_is_directory(world: &mut World) {
    std::fs::create_dir_all(world.state_path()).expect("state path directory");
}

#[when(regex = r#"^the developer presses "([^"]+)"$"#)]
async fn presses(world: &mut World, key: String) {
    world.run_tui_press(&key).await;
}

#[then(regex = r#"^the TUI shows "([^"]+)"$"#)]
async fn shows(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected TUI to contain {expected:?}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the TUI marks "([^"]+)" on "([^"]+)" as approximate$"#)]
async fn marks_approximate(world: &mut World, summary: String, key: String) {
    let marked = world
        .last_transcript
        .lines()
        .any(|line| line.contains(&summary) && line.contains('~'));
    assert!(
        marked,
        "no approximate activity row for {summary:?} on {key}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the TUI does not show "([^"]+)"$"#)]
async fn does_not_show(world: &mut World, unexpected: String) {
    assert!(
        !world.last_transcript.contains(&unexpected),
        "unexpected TUI text {unexpected:?}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the TUI reports "([^"]+)"$"#)]
async fn reports(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected TUI report {expected:?}: {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the selected activity is "([^"]+)"$"#)]
async fn selected(world: &mut World, summary: String) {
    world.assert_selected_activity(&summary);
}

#[then(regex = r#"^the activity "([^"]+)" is (read|unread)$"#)]
async fn activity_read(world: &mut World, summary: String, state: String) {
    world.assert_activity_read(&summary, state == "read").await;
}

#[then(regex = r#"^the terminal UI is closed$"#)]
async fn closed(world: &mut World) {
    assert!(world.tui_closed, "terminal UI was not closed");
}

#[when(regex = r#"^the browser will fail to open$"#)]
async fn browser_fails(world: &mut World) {
    world.fail_browser();
}

#[then(regex = r#"^the browser opened "([^"]+)"$"#)]
async fn browser_opened(world: &mut World, url: String) {
    let opened = world.browser_urls();
    assert!(
        opened.iter().any(|candidate| candidate == &url),
        "expected the browser to open {url:?}, got {opened:?}"
    );
}
