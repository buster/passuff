use cucumber::{then, when};
use passuff_config::WatchMode;

use crate::support::world::{key, World};

#[when(expr = "the developer opens the inbox TUI")]
fn opens_inbox(world: &mut World) {
    world.tui_open(&[], true);
}

#[when(expr = "the developer opens the inbox TUI with keys {string}")]
fn opens_inbox_with_keys(world: &mut World, keys: String) {
    let parsed = passuff::tui::keys::parse_keys(&keys).expect("keys");
    world.tui_open(&parsed, true);
}

#[when(expr = "the developer opens the inbox TUI observing the account")]
fn opens_inbox_observing(world: &mut World) {
    world.tui_open(&[], true);
}

#[when(expr = "the developer opens the inbox TUI and lets it observe for {string} seconds")]
fn opens_inbox_observe_for(world: &mut World, _seconds: String) {
    world.tui_open(&[], true);
}

#[when(expr = "the developer opens the inbox TUI without a terminal")]
fn opens_inbox_without_terminal(world: &mut World) {
    world.run_tui_command(false);
}

#[when(expr = "the developer opens the inbox TUI without a snapshot")]
fn opens_inbox_without_snapshot(world: &mut World) {
    world.run_tui_command(false);
}

#[when(expr = "the developer starts the inbox TUI without a snapshot")]
fn starts_inbox_without_snapshot(world: &mut World) {
    world.run_tui_command(false);
}

#[when(expr = "the developer starts the inbox TUI binary")]
fn starts_tui_binary(world: &mut World) {
    world.run_tui_command(true);
}

#[when(expr = "the developer presses {string}")]
fn developer_presses(world: &mut World, key_name: String) {
    let keys = passuff::tui::keys::parse_keys(&key_name).expect("key");
    for key in keys {
        world.tui_press(key);
    }
}

#[when(expr = "the developer presses {string} in the scope picker")]
fn developer_presses_in_picker(world: &mut World, key_name: String) {
    let keys = passuff::tui::keys::parse_keys(&key_name).expect("key");
    for key in keys {
        world.tui_press(key);
    }
}

#[when(expr = "the developer moves the selection down {int} times")]
fn moves_selection_down(world: &mut World, times: usize) {
    for _ in 0..times {
        world.tui_press(passuff::tui::keys::Key::Down);
    }
}

#[when(expr = "the developer opens the scope picker")]
fn opens_scope_picker(world: &mut World) {
    world.open_picker();
}

#[when(expr = "the developer opens the live scope picker")]
fn opens_live_scope_picker(world: &mut World) {
    if world.tui_app.is_none() {
        world.build_tui(&[]);
    }
    world.tui_press(passuff::tui::keys::Key::OpenPicker);
}

#[when(expr = "the developer refreshes the inbox")]
fn refreshes_inbox(world: &mut World) {
    world.tui_refresh();
}

#[when(expr = "the developer selects {string} in the watch scope")]
fn selects_repository(world: &mut World, repository: String) {
    match world.select_repository(&repository) {
        Ok(()) => world.open_picker(),
        Err(error) => {
            world.tui_exit = 2;
            world.tui_output = format!("passuff-tui: {error}\n");
        }
    }
}

#[when(expr = "the developer deselects {string} in the watch scope")]
fn deselects_repository(world: &mut World, repository: String) {
    match world.deselect_repository(&repository) {
        Ok(()) => world.open_picker(),
        Err(error) => {
            world.tui_exit = 2;
            world.tui_output = format!("passuff-tui: {error}\n");
        }
    }
}

#[when(expr = "the developer switches the watch scope to {string}")]
fn switches_scope(world: &mut World, mode: String) {
    let mode = if mode == "selected" {
        WatchMode::Selected
    } else {
        WatchMode::All
    };
    if let Err(error) = world.set_scope_mode(mode) {
        world.tui_exit = 2;
        world.tui_output = format!("passuff-tui: {error}\n");
    }
}

#[when(expr = "the developer narrows the focus to {string}")]
fn narrows_focus(world: &mut World, relations: String) {
    let parsed: Vec<_> = relations
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter_map(passuff_config::FocusRelation::parse)
        .collect();
    world.set_focus(&parsed);
    world.open_picker();
}

#[then(expr = "the TUI shows {string}")]
fn tui_shows(world: &mut World, expected: String) {
    assert!(
        world.tui_output.contains(&expected),
        "expected TUI to show {expected:?}, got:\n{}",
        world.tui_output
    );
}

#[then(expr = "the TUI does not show {string}")]
fn tui_does_not_show(world: &mut World, unexpected: String) {
    assert!(
        !world.tui_output.contains(&unexpected),
        "expected TUI not to show {unexpected:?}, got:\n{}",
        world.tui_output
    );
}

#[then(expr = "the TUI reports {string}")]
fn tui_reports(world: &mut World, expected: String) {
    assert!(
        world.tui_output.contains(&expected),
        "expected TUI to report {expected:?}, got:\n{}",
        world.tui_output
    );
}

#[then(expr = "the TUI shows {string} before {string}")]
fn tui_shows_before(world: &mut World, first: String, second: String) {
    let first_position = world
        .tui_output
        .find(&first)
        .unwrap_or_else(|| panic!("missing {first:?} in:\n{}", world.tui_output));
    let second_position = world
        .tui_output
        .find(&second)
        .unwrap_or_else(|| panic!("missing {second:?} in:\n{}", world.tui_output));
    assert!(
        first_position < second_position,
        "expected {first:?} before {second:?}"
    );
}

#[then(expr = "the TUI marks {string} on {string} as approximate")]
fn tui_marks_approximate(world: &mut World, summary: String, key_value: String) {
    let found = world
        .tui_output
        .lines()
        .any(|line| line.contains(&summary) && line.contains(&key_value) && line.contains('~'));
    assert!(
        found,
        "expected approximate {summary:?} on {key_value}:\n{}",
        world.tui_output
    );
}

#[then(expr = "the selected activity is {string}")]
fn selected_activity(world: &mut World, expected: String) {
    assert_eq!(
        world.selected_activity_summary().as_deref(),
        Some(expected.as_str())
    );
}

#[then(expr = "the selected pull request is {string}")]
fn selected_pull_request(world: &mut World, expected: String) {
    let selected = world
        .selected_pr_key()
        .unwrap_or_else(|| panic!("no selected pull request"));
    assert_eq!(selected, key(&expected));
}

#[then(expr = "the terminal UI is closed")]
fn terminal_ui_closed(world: &mut World) {
    assert!(
        world.tui_closed || world.tui_exit == 0,
        "expected terminal UI to be closed (exit {})",
        world.tui_exit
    );
}

#[then(expr = "the TUI still shows {string}")]
fn tui_still_shows(world: &mut World, expected: String) {
    assert!(
        world.tui_output.contains(&expected),
        "expected TUI to still show {expected:?}, got:\n{}",
        world.tui_output
    );
}

#[then(expr = "the picker shows {string}")]
fn picker_shows(world: &mut World, expected: String) {
    assert!(
        world.picker_output.contains(&expected) || world.tui_output.contains(&expected),
        "expected picker to show {expected:?}, got:\n{}",
        world.picker_output
    );
}

#[then(expr = "the picker does not show {string}")]
fn picker_does_not_show(world: &mut World, unexpected: String) {
    assert!(
        !world.picker_output.contains(&unexpected) && !world.tui_output.contains(&unexpected),
        "expected picker not to show {unexpected:?}, got:\n{}",
        world.picker_output
    );
}

#[then(expr = "the notification sound is muted")]
fn notification_sound_muted(world: &mut World) {
    assert!(!world.ui_sound_enabled(), "expected sound to be muted");
}

#[then(expr = "the desktop notification is enabled")]
fn desktop_notification_enabled(world: &mut World) {
    assert!(
        world.ui_desktop_enabled(),
        "expected desktop notifications on"
    );
}
