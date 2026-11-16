//! CLI steps: every action starts the built binary in a fresh process.

use cucumber::{given, then, when};

use crate::harness::world::World;

#[given(expr = "the built product and a mock GitHub account")]
fn built_product(world: &mut World) {
    world.assert_ready();
}

#[given(expr = "a mock GitHub account with desktop notifications enabled")]
fn desktop_notifications(world: &mut World) {
    world.enable_desktop();
}

#[given(expr = "a mock GitHub account that answers every operation with a server error")]
fn server_error(world: &mut World) {
    world.fail_everything();
}

#[given(expr = "a mock GitHub account pointed at a closed port")]
fn closed_port(world: &mut World) {
    world.use_closed_port();
}

#[when(expr = "the developer asks for help")]
fn asks_for_help(world: &mut World) {
    world.run(&["--help"]).expect("run passuff --help");
}

#[when(expr = "the developer runs one check")]
fn runs_one_check(world: &mut World) {
    world.run(&["check"]).expect("run passuff check");
}

#[when(expr = "the developer hides new activity from the authored query")]
fn hides_from_authored(world: &mut World) {
    world.observe_only();
}

#[when(expr = "the developer adds a comment on the watched pull request")]
fn adds_comment(world: &mut World) {
    world.add_comment();
}

#[when(expr = "the developer asks for status")]
fn asks_for_status(world: &mut World) {
    world.run(&["status"]).expect("run passuff status");
}

#[when(expr = "the developer lists the inbox")]
fn lists_inbox(world: &mut World) {
    world.run(&["inbox"]).expect("run passuff inbox");
}

#[when(expr = "the developer marks all activity read")]
fn marks_all_read(world: &mut World) {
    world
        .run(&["read", "--all"])
        .expect("run passuff read --all");
}

#[then(expr = "the product exits with code {int}")]
fn product_exit(world: &mut World, expected: i32) {
    world.assert_exit(expected);
}

#[then(expr = "the output contains {string}")]
fn output_contains(world: &mut World, needle: String) {
    world.assert_output(&needle);
}

#[then(expr = "the error output contains {string}")]
fn error_output_contains(world: &mut World, needle: String) {
    world.assert_error_output(&needle);
}

#[then(expr = "the notification log holds one line titled {string}")]
fn notification_line(world: &mut World, title: String) {
    world.assert_notification_line(&title);
}
