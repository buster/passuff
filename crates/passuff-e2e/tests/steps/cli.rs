//! CLI steps: every action starts the built binary in a fresh process.

use cucumber::{gherkin::Step, given, then, when};

use crate::harness::bin::TOKEN;
use crate::harness::world::{World, CORRUPT_STATE, VERSION_ONE_STATE};

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

#[given(expr = "no configuration file exists")]
fn no_configuration(world: &mut World) {
    world.remove_config();
}

#[given(expr = "an existing configuration file")]
fn existing_configuration(world: &mut World) {
    world.remember_config();
}

#[given(expr = "the viewer login is {string}")]
fn viewer_login(world: &mut World, login: String) {
    world.set_viewer_login(&login);
}

#[given(expr = "a second configured account")]
fn second_account(world: &mut World) {
    world.add_personal_account();
}

#[given(expr = "the account reads its token from a file with mode {word}")]
fn token_file(world: &mut World, mode: String) {
    let mode = u32::from_str_radix(&mode, 8)
        .unwrap_or_else(|error| panic!("token file mode {mode:?}: {error}"));
    world.use_token_file(TOKEN, mode);
}

#[given(expr = "bot comments are enabled")]
fn bot_comments(world: &mut World) {
    world.enable_bot_comments();
}

#[given(expr = "a version-1 state file with one watched pull request")]
fn version_one_state(world: &mut World) {
    world.write_state(VERSION_ONE_STATE);
}

#[given(expr = "a corrupt state file")]
fn corrupt_state(world: &mut World) {
    world.write_state(CORRUPT_STATE);
}

#[when(expr = "the developer asks for help")]
fn asks_for_help(world: &mut World) {
    world.run(&["--help"]).expect("run passuff --help");
}

#[when(expr = "the developer runs one check")]
fn runs_one_check(world: &mut World) {
    world.run(&["check"]).expect("run passuff check");
}

#[when(expr = "the developer runs setup with:")]
fn runs_setup(world: &mut World, step: &Step) {
    let answers = step.docstring().expect("the setup answers");
    world
        .run_with_input(&["setup"], &format!("{answers}\n"))
        .expect("run passuff setup");
}

#[when(expr = "the developer runs setup and declines replacement")]
fn declines_replacement(world: &mut World) {
    world
        .run_with_input(&["setup"], "n\n")
        .expect("run passuff setup");
}

#[when(expr = "the developer starts watching")]
fn starts_watching(world: &mut World) {
    world.start_watch().expect("start the watch");
}

#[when(expr = "the developer waits for the baseline")]
fn waits_for_baseline(world: &mut World) {
    world.wait_for_watch_baseline();
}

#[when(expr = "the developer interrupts the watch")]
fn interrupts_watch(world: &mut World) {
    world.interrupt_watch();
}

#[when(expr = "the developer hides new activity from the authored query")]
fn hides_from_authored(world: &mut World) {
    world.observe_only();
}

#[when(expr = "the developer adds a comment on the watched pull request")]
fn adds_comment(world: &mut World) {
    world.add_comment();
}

#[when(expr = "the developer adds a comment of their own")]
fn adds_own_comment(world: &mut World) {
    world.add_own_comment();
}

#[when(expr = "the developer adds a bot comment")]
fn adds_bot_comment(world: &mut World) {
    world.add_bot_comment();
}

#[when(expr = "the developer adds a comment on the second account's pull request")]
fn adds_personal_comment(world: &mut World) {
    world.add_personal_comment();
}

#[when(expr = "the developer makes the pull request CI fail")]
fn fails_ci(world: &mut World) {
    world.fail_ci();
}

#[when(expr = "the developer makes the pull request CI pass")]
fn passes_ci(world: &mut World) {
    world.pass_ci();
}

#[when(expr = "the developer switches the watch scope to {string}")]
fn switches_scope(world: &mut World, repository: String) {
    world.set_watch_scope("selected", &[repository.as_str()]);
}

#[when(expr = "the developer asks for status")]
fn asks_for_status(world: &mut World) {
    world.run(&["status"]).expect("run passuff status");
}

#[when(expr = "the developer lists the inbox")]
fn lists_inbox(world: &mut World) {
    world.run(&["inbox"]).expect("run passuff inbox");
}

#[when(expr = "the developer lists all activity")]
fn lists_all_activity(world: &mut World) {
    world
        .run(&["inbox", "--all"])
        .expect("run passuff inbox --all");
}

#[when(expr = "the developer marks all activity read")]
fn marks_all_read(world: &mut World) {
    world
        .run(&["read", "--all"])
        .expect("run passuff read --all");
}

#[when(expr = "the developer marks the first activity read")]
fn marks_first_read(world: &mut World) {
    world.run(&["read", "a1"]).expect("run passuff read a1");
}

#[when(expr = "the developer runs read with no argument")]
fn read_without_argument(world: &mut World) {
    world.run(&["read"]).expect("run passuff read");
}

#[when(expr = "the developer runs read with an id and --all")]
fn read_with_id_and_all(world: &mut World) {
    world
        .run(&["read", "a1", "--all"])
        .expect("run passuff read a1 --all");
}

#[when(expr = "the developer reads the unknown activity {string}")]
fn reads_unknown_activity(world: &mut World, id: String) {
    world
        .run(&["read", &id])
        .unwrap_or_else(|error| panic!("run passuff read {id}: {error}"));
}

#[then(expr = "the product exits with code {int}")]
fn product_exit(world: &mut World, expected: i32) {
    world.assert_exit(expected);
}

#[then(expr = "the output contains {string}")]
fn output_contains(world: &mut World, needle: String) {
    world.assert_output(&needle);
}

#[then(expr = "the output does not contain {string}")]
fn output_lacks(world: &mut World, needle: String) {
    world.assert_output_lacks(&needle);
}

#[then(expr = "the output contains {int} lines matching {string}")]
fn output_lines(world: &mut World, expected: usize, needle: String) {
    world.assert_output_lines(&needle, expected);
}

#[then(expr = "the error output contains {string}")]
fn error_output_contains(world: &mut World, needle: String) {
    world.assert_error_output(&needle);
}

#[then(expr = "the configuration file is created")]
fn configuration_created(world: &mut World) {
    assert!(
        world.config.is_file(),
        "the configuration file was not created: {}",
        world.config.display()
    );
}

#[then(expr = "the configuration file contains {string}")]
fn configuration_contains(world: &mut World, needle: String) {
    world.assert_config_contains(&needle);
}

#[then(expr = "the configuration file is unchanged")]
fn configuration_unchanged(world: &mut World) {
    world.assert_config_unchanged();
}

#[then(expr = "the state file is version {int}")]
fn state_version(world: &mut World, expected: u64) {
    world.assert_state_version(expected);
}

#[then(expr = "the state file is absent")]
fn state_absent(world: &mut World) {
    world.assert_state_absent();
}

#[then(expr = "the state file contains {string}")]
fn state_contains(world: &mut World, needle: String) {
    world.assert_state_contains(&needle);
}

#[then(expr = "a quarantined state file holds the corrupt bytes")]
fn quarantine_holds_corrupt_bytes(world: &mut World) {
    world.assert_quarantined(CORRUPT_STATE);
}

#[then(expr = "the notification log holds one line titled {string}")]
fn notification_line(world: &mut World, title: String) {
    world.assert_notification_line(&title);
}

#[then(expr = "the watch exits with code {int}")]
fn watch_exit(world: &mut World, expected: i32) {
    world.assert_watch_exit(expected);
}
