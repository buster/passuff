use std::collections::BTreeMap;

use clap::CommandFactory;
use cucumber::{given, then, when};
use passuff::args::Cli;
use passuff_config::WatchMode;
use passuff_core::PullRequestKey;

use crate::support::world::{key, World};

#[when(expr = "passuff runs one check")]
fn passuff_runs_one_check(world: &mut World) {
    world.run_check();
}

#[given(expr = "passuff runs one check")]
fn passuff_runs_one_check_given(world: &mut World) {
    world.run_check();
}

#[when(expr = "the developer runs one check")]
fn developer_runs_one_check(world: &mut World) {
    world.run_check();
}

#[when(expr = "passuff starts watching")]
fn passuff_starts_watching(world: &mut World) {
    world.run_watch_start();
}

#[when(expr = "passuff restarts and runs one check")]
fn passuff_restarts(world: &mut World) {
    world.watcher_running = false;
    world.reload_config();
    world.run_check();
}

#[when(expr = "the developer asks for status")]
fn developer_asks_for_status(world: &mut World) {
    world.run_status();
}

#[when(expr = "the developer lists the inbox")]
fn developer_lists_inbox(world: &mut World) {
    world.run_inbox(false);
}

#[when(expr = "the developer lists all activity")]
fn developer_lists_all_activity(world: &mut World) {
    world.run_inbox(true);
}

#[when(expr = "the developer marks the newest activity read")]
fn developer_marks_newest_read(world: &mut World) {
    if let Some(id) = world.newest_activity_id() {
        world.run_read(Some(&id), false);
    }
}

#[when(expr = "the developer marks all activity read")]
fn developer_marks_all_read(world: &mut World) {
    world.run_read(None, true);
}

#[when(expr = "the developer marks activity {string} read")]
fn developer_marks_activity_read(world: &mut World, id: String) {
    world.run_read(Some(&id), false);
}

#[when(expr = "the developer runs read with no target")]
fn developer_runs_read_no_target(world: &mut World) {
    world.run_read(None, false);
}

#[when(expr = "the developer runs the notification test")]
fn developer_runs_notification_test(world: &mut World) {
    world.run_notify_test(false, false);
}

#[when(expr = "the developer runs the notification test with {string}")]
fn developer_runs_notification_test_with(world: &mut World, flag: String) {
    world.run_notify_test(flag == "--no-desktop", flag == "--no-sound");
}

#[when(expr = "the developer runs passuff with {string}")]
fn developer_runs_passuff_with(world: &mut World, argument: String) {
    let mut command = Cli::command();
    let text = if argument == "--version" {
        command.render_version().to_string()
    } else {
        command.render_help().to_string()
    };
    world.last_exit = 0;
    world.transcripts.push(text);
}

#[when(expr = "the developer reopens passuff")]
fn developer_reopens(world: &mut World) {
    world.reload_config();
}

#[when(expr = "the watcher starts interactively")]
fn watcher_starts_interactively(world: &mut World) {
    world.interactive = true;
    world.run_watch_start();
}

#[when(expr = "the watcher runs one observation cycle")]
fn watcher_runs_one_cycle(world: &mut World) {
    world.run_watch_cycle();
}

#[when(expr = "the watcher runs two observation cycles")]
fn watcher_runs_two_cycles(world: &mut World) {
    world.run_watch_cycle_twice();
}

#[when(expr = "the watcher runs another cycle")]
fn watcher_runs_another_cycle(world: &mut World) {
    world.run_watch_cycle();
}

#[when(expr = "the inbox plans its next observation")]
fn inbox_plans_next_observation(world: &mut World) {
    world.plan_next_delay();
}

#[when(expr = "the inbox observes the account")]
fn inbox_observes_account(world: &mut World) {
    world.tui_observe();
}

#[when(expr = "the inbox observes the account three times with no change")]
fn inbox_observes_three_times(world: &mut World) {
    world.tui_observe();
    world.tui_observe();
    world.tui_observe();
}

#[when(expr = "the inbox waits {string} seconds")]
fn inbox_waits(world: &mut World, seconds: String) {
    world.clock.advance_seconds(seconds.parse().unwrap_or(0));
}

#[when(expr = "the developer runs setup with:")]
fn developer_runs_setup_with(world: &mut World, step: &cucumber::gherkin::Step) {
    let mut answers = BTreeMap::new();
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            if row.len() >= 2 {
                answers.insert(row[0].trim().to_string(), row[1].trim().to_string());
            }
        }
    }
    let mut input = String::new();
    if answers.get("replace").map(String::as_str) == Some("yes") {
        input.push_str("yes\n");
    }
    input.push_str(
        answers
            .get("label")
            .map(String::as_str)
            .unwrap_or("personal"),
    );
    input.push('\n');
    input.push_str(
        answers
            .get("auth")
            .map(String::as_str)
            .unwrap_or("gh_oauth"),
    );
    input.push('\n');
    input.push_str(
        answers
            .get("login")
            .map(String::as_str)
            .unwrap_or("octo-personal"),
    );
    input.push('\n');
    match answers
        .get("auth")
        .map(String::as_str)
        .unwrap_or("gh_oauth")
    {
        "token_env" => {
            input.push_str(
                answers
                    .get("variable")
                    .map(String::as_str)
                    .unwrap_or("WORK_TOKEN"),
            );
            input.push('\n');
        }
        "token_command" => {
            input.push_str(
                answers
                    .get("command")
                    .map(String::as_str)
                    .unwrap_or("echo token"),
            );
            input.push('\n');
        }
        "token_file" => {
            let file = answers
                .get("file")
                .map(|value| {
                    if value == "TOKEN_FILE" {
                        world
                            .token_file
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| value.clone())
                    } else {
                        value.clone()
                    }
                })
                .unwrap_or_default();
            input.push_str(&file);
            input.push('\n');
        }
        _ => {}
    }
    input.push_str("no\n");
    input.push_str(answers.get("watch").map(String::as_str).unwrap_or("all"));
    input.push('\n');
    if answers.get("watch").map(String::as_str) == Some("selected") {
        input.push_str(answers.get("repos").map(String::as_str).unwrap_or(""));
        input.push('\n');
    }
    world.run_setup(&input);
}

#[when(expr = "the developer runs setup and answers {string} to replacement")]
fn developer_runs_setup_replacement(world: &mut World, answer: String) {
    let input = format!("{answer}\n");
    world.run_setup(&input);
}

#[then(expr = "the transcript reports {string}")]
fn transcript_reports(world: &mut World, expected: String) {
    let transcript = world.latest();
    assert!(
        transcript.contains(&expected),
        "expected transcript to contain {expected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the transcript does not report {string}")]
fn transcript_does_not_report(world: &mut World, unexpected: String) {
    let transcript = world.latest();
    assert!(
        !transcript.contains(&unexpected),
        "expected transcript not to contain {unexpected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the latest transcript reports {string}")]
fn latest_transcript_reports(world: &mut World, expected: String) {
    let transcript = world.latest();
    assert!(
        transcript.contains(&expected),
        "expected transcript to contain {expected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the terminal shows a notification for {string}")]
fn terminal_shows_notification_for(world: &mut World, key_value: String) {
    let transcript = world.latest();
    let expected = format!("notification for {key_value}");
    assert!(
        transcript.contains(&expected),
        "expected {expected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the transcript reports a notification for {string}")]
fn transcript_notification_for(world: &mut World, key_value: String) {
    let transcript = world.latest();
    let expected = format!("notification for {key_value}");
    assert!(
        transcript.contains(&expected),
        "expected {expected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the transcript reports no notification for {string}")]
fn transcript_no_notification_for(world: &mut World, key_value: String) {
    let transcript = world.latest();
    let expected = format!("notification for {key_value}");
    assert!(
        !transcript.contains(&expected),
        "expected no {expected:?}, got:\n{transcript}"
    );
}

#[then(expr = "the transcript reports one notification for {string}")]
fn transcript_one_notification_for(world: &mut World, key_value: String) {
    let transcript = world.latest();
    let expected = format!("notification for {key_value}");
    let count = transcript.matches(&expected).count();
    assert_eq!(count, 1, "expected one {expected:?}, got:\n{transcript}");
}

#[then(expr = "the notification mentions {string}, {string}, and {string}")]
fn notification_mentions(world: &mut World, first: String, second: String, third: String) {
    let transcript = world.latest();
    for expected in [first, second, third] {
        assert!(
            transcript.contains(&expected),
            "expected {expected:?}, got:\n{transcript}"
        );
    }
}

#[then(expr = "the desktop notification reports {string}")]
fn desktop_notification_reports(world: &mut World, expected: String) {
    let messages = world.delivery.messages();
    assert!(
        messages
            .iter()
            .any(|message| message.title.contains(&expected) || message.body.contains(&expected)),
        "expected desktop notification {expected:?}, got: {messages:?}"
    );
}

#[then(expr = "the desktop notification is recorded")]
fn desktop_notification_recorded(world: &mut World) {
    assert!(
        !world.delivery.messages().is_empty(),
        "expected a desktop notification"
    );
}

#[then(expr = "no desktop notification is recorded")]
fn no_desktop_notification(world: &mut World) {
    assert!(
        world.delivery.messages().is_empty(),
        "expected no desktop notification, got: {:?}",
        world.delivery.messages()
    );
}

#[then(expr = "the configured sound is played once")]
fn configured_sound_played_once(world: &mut World) {
    let plays = world.sound.plays();
    assert_eq!(plays.len(), 1, "expected one sound, got: {plays:?}");
}

#[then(expr = "the configured sound file {string} is played once")]
fn configured_sound_file_played_once(world: &mut World, path: String) {
    let plays = world.sound.plays();
    assert_eq!(plays.len(), 1, "expected one sound, got: {plays:?}");
    let requested = std::path::Path::new(&path);
    assert!(
        plays[0].ends_with(requested) || plays[0].file_name() == requested.file_name(),
        "expected sound {path:?}, got: {plays:?}"
    );
}

#[then(expr = "no sound is played")]
fn no_sound_played(world: &mut World) {
    assert!(
        world.sound.plays().is_empty(),
        "expected no sound, got: {:?}",
        world.sound.plays()
    );
}

#[then(expr = "the exit status is {int}")]
fn exit_status(world: &mut World, expected: i32) {
    assert_eq!(
        world.last_exit,
        expected,
        "expected exit {expected}, got {}. transcript:\n{}",
        world.last_exit,
        world.latest()
    );
}

#[then(expr = "the state retains {string}")]
fn state_retains(world: &mut World, key_value: String) {
    let state = world.state();
    assert!(
        state
            .accounts
            .values()
            .any(|account| account.pull_requests.contains_key(&key_value)),
        "expected state to retain {key_value}"
    );
}

#[then(expr = "the state retains {int} activity records")]
fn state_retains_activity(world: &mut World, expected: usize) {
    assert_eq!(
        world.state().activity.len(),
        expected,
        "unexpected activity count"
    );
}

#[then(expr = "the oldest activity record is gone")]
fn oldest_activity_gone(world: &mut World) {
    let state = world.state();
    assert!(
        !state.activity.iter().any(|record| record.id == "a1"),
        "expected a1 to be dropped"
    );
}

#[then(expr = "the state file version is {int}")]
fn state_file_version(world: &mut World, expected: u64) {
    let text = std::fs::read_to_string(&world.paths.state).expect("state file");
    let value: serde_json::Value = serde_json::from_str(&text).expect("state json");
    assert_eq!(
        value.get("version").and_then(serde_json::Value::as_u64),
        Some(expected)
    );
}

#[then(expr = "no state file was created")]
fn no_state_file_created(world: &mut World) {
    assert!(
        !world.paths.state.exists(),
        "expected no state file at {}",
        world.paths.state.display()
    );
}

#[then(expr = "the report records a baseline for {string}")]
fn report_baseline(world: &mut World, key_value: String) {
    assert!(
        world.baseline_recorded_for(&key(&key_value)),
        "expected baseline for {key_value}"
    );
}

#[then(expr = "the activity {string} is {word}")]
fn activity_read_state(world: &mut World, summary: String, state: String) {
    let read = world
        .activity_is_read(&summary)
        .unwrap_or_else(|| panic!("activity {summary:?} not found"));
    assert_eq!(read, state == "read", "expected {summary:?} to be {state}");
}

#[then(expr = "every inbox record is unread")]
fn every_inbox_record_unread(world: &mut World) {
    let state = world.state();
    assert!(
        state.activity.iter().all(|record| !record.read),
        "expected every activity record to be unread"
    );
}

#[then(expr = "the inbox shows an activity id and occurrence time")]
fn inbox_shows_id_and_time(world: &mut World) {
    let transcript = world.latest();
    assert!(
        transcript.contains("a1") || transcript.contains("a2"),
        "expected an activity id in:\n{transcript}"
    );
    assert!(
        transcript.contains("2026-11-02T"),
        "expected an occurrence time in:\n{transcript}"
    );
}

#[then(expr = "the inbox shows {string} before {string} for {string}")]
fn inbox_shows_before(world: &mut World, first: String, second: String, key_value: String) {
    let transcript = world.latest();
    let first_position = transcript
        .find(&first)
        .unwrap_or_else(|| panic!("missing {first:?} in:\n{transcript}"));
    let second_position = transcript
        .find(&second)
        .unwrap_or_else(|| panic!("missing {second:?} in:\n{transcript}"));
    assert!(
        first_position < second_position && transcript.contains(&key_value),
        "expected {first:?} before {second:?} for {key_value}:\n{transcript}"
    );
}

#[then(expr = "the inbox shows {string} for {string} as {word}")]
fn inbox_shows_state(world: &mut World, summary: String, key_value: String, state: String) {
    let transcript = world.latest();
    let found = transcript
        .lines()
        .any(|line| line.contains(&summary) && line.contains(&key_value) && line.contains(&state));
    assert!(
        found,
        "expected {summary:?} for {key_value} as {state}:\n{transcript}"
    );
}

#[then(expr = "the inbox marks {string} on {string} as approximate")]
fn inbox_marks_approximate(world: &mut World, summary: String, key_value: String) {
    let transcript = world.latest();
    let found = transcript
        .lines()
        .any(|line| line.contains(&summary) && line.contains(&key_value) && line.contains('~'));
    assert!(
        found,
        "expected approximate {summary:?} on {key_value}:\n{transcript}"
    );
}

#[then(expr = "the inbox shows the occurrence time {string} for {string}")]
fn inbox_shows_occurrence(world: &mut World, time: String, key_value: String) {
    let transcript = world.latest();
    let found = transcript
        .lines()
        .any(|line| line.contains(&time) && line.contains(&key_value));
    assert!(
        found,
        "expected occurrence {time:?} for {key_value}:\n{transcript}"
    );
}

#[then(expr = "the credential value {string} appears in neither the transcript nor the state file")]
fn credential_not_in_artifacts(world: &mut World, value: String) {
    for transcript in &world.transcripts {
        assert!(
            !transcript.contains(&value),
            "credential appeared in transcript"
        );
    }
    assert!(
        !world.state_text().contains(&value),
        "credential appeared in state"
    );
    assert!(
        !world.config_text().contains(&value),
        "credential appeared in configuration"
    );
}

#[then(expr = "the configuration contains account {string}")]
fn config_contains_account(world: &mut World, label: String) {
    let text = world.config_text();
    assert!(
        text.contains(&format!("[accounts.{label}]"))
            || text.contains(&format!("accounts.{label}")),
        "expected account {label} in:\n{text}"
    );
}

#[then(expr = "the configuration contains auth {string}")]
fn config_contains_auth(world: &mut World, auth: String) {
    let text = world.config_text();
    assert!(
        text.contains(&format!("auth = \"{auth}\"")),
        "expected auth {auth} in:\n{text}"
    );
}

#[then(expr = "the configuration contains token env {string}")]
fn config_contains_token_env(world: &mut World, variable: String) {
    let text = world.config_text();
    assert!(
        text.contains(&format!("token_env = \"{variable}\"")),
        "expected token env {variable} in:\n{text}"
    );
}

#[then(expr = "the configuration contains login {string}")]
fn config_contains_login(world: &mut World, login: String) {
    let text = world.config_text();
    assert!(
        text.contains(&format!("github_user = \"{login}\"")),
        "expected login {login} in:\n{text}"
    );
}

#[then(expr = "the configuration does not contain {string}")]
fn config_does_not_contain(world: &mut World, value: String) {
    let text = world.config_text();
    assert!(
        !text.contains(&value),
        "expected configuration not to contain {value:?}:\n{text}"
    );
}

#[then(expr = "the configuration is created")]
fn config_created(world: &mut World) {
    assert!(
        world.paths.config.exists(),
        "expected configuration at {}",
        world.paths.config.display()
    );
}

#[then(expr = "the configuration is not created")]
fn config_not_created(world: &mut World) {
    assert!(
        !world.paths.config.exists(),
        "expected no configuration at {}",
        world.paths.config.display()
    );
}

#[then(expr = "the configuration is unchanged")]
fn config_unchanged(world: &mut World) {
    let before = world.config_before.clone().unwrap_or_default();
    assert_eq!(
        world.config_text(),
        before,
        "expected configuration to be unchanged"
    );
}

#[then(expr = "setup reports {string}")]
fn setup_reports(world: &mut World, expected: String) {
    assert!(
        world.setup_output.contains(&expected) || world.latest().contains(&expected),
        "expected setup to report {expected:?}, got:\n{}",
        world.setup_output
    );
}

#[then(expr = "the watch scope mode is {string}")]
fn then_watch_scope_mode(world: &mut World, expected: String) {
    world.reload_config();
    assert_eq!(world.config.watch.effective_mode().as_str(), expected);
}

#[then(expr = "the watch scope contains {string}")]
fn then_watch_scope_contains(world: &mut World, repository: String) {
    world.reload_config();
    assert!(
        world
            .config
            .watch
            .repositories
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&repository)),
        "expected watch scope to contain {repository}"
    );
}

#[then(expr = "the watch scope does not contain {string}")]
fn then_watch_scope_does_not_contain(world: &mut World, repository: String) {
    world.reload_config();
    assert!(
        !world
            .config
            .watch
            .repositories
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&repository)),
        "expected watch scope not to contain {repository}"
    );
}

#[then(expr = "the watch scope is empty")]
fn then_watch_scope_empty(world: &mut World) {
    world.reload_config();
    assert!(world.config.watch.repositories.is_empty());
}

#[then(expr = "the focus relations are {string}")]
fn then_focus_relations(world: &mut World, expected: String) {
    world.reload_config();
    let expected: Vec<&str> = expected
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let actual: Vec<&str> = world
        .config
        .focus
        .relations
        .iter()
        .map(|relation| relation.as_str())
        .collect();
    assert_eq!(actual, expected);
}

#[then(expr = "gh started a browser login for {string}")]
fn gh_started_login(world: &mut World, login: String) {
    let attempts = world
        .data
        .lock()
        .map(|data| data.gh_login_attempts.clone())
        .unwrap_or_default();
    assert!(
        attempts.contains(&login),
        "expected login attempt for {login}, got {attempts:?}"
    );
}

#[then(expr = "gh started one browser login for {string}")]
fn gh_started_one_login(world: &mut World, login: String) {
    let attempts = world
        .data
        .lock()
        .map(|data| data.gh_login_attempts.clone())
        .unwrap_or_default();
    assert_eq!(
        attempts.iter().filter(|attempt| *attempt == &login).count(),
        1,
        "expected one login attempt for {login}, got {attempts:?}"
    );
}

#[then(expr = "gh did not start a browser login")]
fn gh_did_not_start_login(world: &mut World) {
    let attempts = world
        .data
        .lock()
        .map(|data| data.gh_login_attempts.clone())
        .unwrap_or_default();
    assert!(attempts.is_empty(), "expected no login, got {attempts:?}");
}

#[then(expr = "the transcript shows the available commands")]
fn transcript_shows_commands(world: &mut World) {
    let transcript = world.latest();
    assert!(transcript.contains("check") && transcript.contains("status"));
}

#[then(expr = "the transcript shows the version")]
fn transcript_shows_version(world: &mut World) {
    assert!(world.latest().to_ascii_lowercase().contains("passuff"));
}

#[then(expr = "the inbox waits at least {string} seconds before observing again")]
fn inbox_waits_at_least(world: &mut World, seconds: String) {
    let expected: u64 = seconds.parse().unwrap_or(30);
    let delay = world
        .planned_delay
        .expect("the inbox planned no observation");
    assert!(
        delay.as_secs() >= expected,
        "expected at least {expected}s, got {delay:?}"
    );
}

#[then(expr = "the inbox has asked GitHub for the full account history {string} time")]
fn full_history_count(world: &mut World, count: String) {
    let expected: usize = count.parse().unwrap_or(0);
    assert_eq!(world.full_history_calls(), expected);
}

#[then(expr = "the inbox has asked GitHub for a change signal {string} times")]
fn probe_count(world: &mut World, count: String) {
    let expected: usize = count.parse().unwrap_or(0);
    assert_eq!(world.probe_calls(), expected);
}

#[then(expr = "the inbox has observed only {string}")]
fn observed_only(world: &mut World, key_value: String) {
    let observed = world.last_observed_keys();
    assert_eq!(observed, vec![key(&key_value)]);
}

#[then(expr = "the browser opened {string}")]
fn browser_opened(world: &mut World, url: String) {
    assert!(
        world.browser.urls().contains(&url),
        "expected browser to open {url:?}, got {:?}",
        world.browser.urls()
    );
}

#[allow(dead_code)]
fn _unused(_: BTreeMap<String, String>, _: PullRequestKey, _: WatchMode) {}
