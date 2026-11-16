//! TUI steps: the product runs on a pty at the scenario's size; assertions read
//! the parsed screen and the process exit code.

use cucumber::{then, when};

use crate::harness::world::World;

#[when(expr = "the developer opens the inbox at {int}x{int}")]
fn opens_inbox(world: &mut World, cols: u16, rows: u16) {
    world
        .open_inbox(cols, rows)
        .expect("open the inbox on a pty");
}

#[when(expr = "the developer presses enter")]
fn presses_enter(world: &mut World) {
    world.press_enter().expect("press enter");
}

#[when(expr = "the developer quits the inbox")]
fn quits_inbox(world: &mut World) {
    world.quit_inbox().expect("quit the inbox");
}

#[then(expr = "the screen shows {string}")]
fn screen_shows(world: &mut World, needle: String) {
    world.screen_shows(&needle);
}

#[then(expr = "the TUI exits with code {int}")]
fn tui_exit(world: &mut World, expected: i32) {
    world.assert_tui_exit(expected);
}
