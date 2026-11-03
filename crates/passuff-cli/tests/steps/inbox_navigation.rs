use cucumber::{given, then, when};

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
