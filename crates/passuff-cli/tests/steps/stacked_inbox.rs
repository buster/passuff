use cucumber::{given, then, when};

use crate::support::World;

#[given(
    regex = r#"^GitHub reports stack (\d+) for "([^"]+)" at position (\d+) of (\d+) with base "([^"]+)"$"#
)]
async fn reports_stack(
    world: &mut World,
    number: u64,
    pull_request: String,
    position: u64,
    size: u64,
    base: String,
) {
    world.pr_mut(&pull_request).stack = Some(passuff_core::PullRequestStack {
        number,
        size,
        position,
        base: base.clone(),
    });
    if world.e2e {
        world
            .e2e_set_stack(&pull_request, number, position, size, &base)
            .await;
    }
}

#[when(
    regex = r#"^GitHub reports stack (\d+) for "([^"]+)" at position (\d+) of (\d+) with base "([^"]+)"$"#
)]
async fn reports_stack_after_when(
    world: &mut World,
    number: u64,
    pull_request: String,
    position: u64,
    size: u64,
    base: String,
) {
    reports_stack(world, number, pull_request, position, size, base).await;
}

#[then(regex = r#"^the TUI shows "([^"]+)" before "([^"]+)"$"#)]
async fn shows_before(world: &mut World, first: String, second: String) {
    let first_at = world
        .last_transcript
        .find(&first)
        .unwrap_or_else(|| panic!("missing {first:?} in {:?}", world.last_transcript));
    let second_at = world
        .last_transcript
        .find(&second)
        .unwrap_or_else(|| panic!("missing {second:?} in {:?}", world.last_transcript));
    assert!(
        first_at < second_at,
        "expected {first:?} before {second:?} in {:?}",
        world.last_transcript
    );
}
