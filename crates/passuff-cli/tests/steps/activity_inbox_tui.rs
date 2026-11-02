use cucumber::{then, when};

use crate::support::World;

#[when(regex = r#"^the developer opens the inbox TUI$"#)]
async fn opens(world: &mut World) {
    world.run_tui_snapshot().await;
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
