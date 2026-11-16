use std::collections::BTreeMap;

use chrono::Duration;
use cucumber::{given, then, when};
use passuff_config::{AccountConfig, AuthMode, FocusRelation, WatchMode};
use passuff_core::{CiResult, Clock, PullRequestKey, PullRequestState, ReviewState};

use crate::support::fake::FakeCredential;
use crate::support::world::{ci_result, key, parse_time, pr_state, review_state, World};

fn gh_authenticated(world: &mut World, login: &str) {
    if let Ok(mut data) = world.data.lock() {
        data.gh_authenticated.insert(login.to_string());
    }
}

fn set_pr_body(world: &mut World, key_value: &str, body: String) {
    let pull_request_key = key(key_value);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.body = body;
        }
    }
}

fn add_relation(world: &mut World, key_value: &str, relation: FocusRelation) {
    let pull_request_key = key(key_value);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            if !pull_request.relations.contains(&relation) {
                pull_request.relations.push(relation);
            }
        }
    }
}

#[given(expr = "a configured account {string} with login {string}")]
fn configured_account(world: &mut World, label: String, login: String) {
    world.add_gh_account(&label, &login);
    gh_authenticated(world, &login);
}

#[given(expr = "a configured account {string} authenticating through gh as {string}")]
fn configured_gh_account(world: &mut World, label: String, login: String) {
    world.add_gh_account(&label, &login);
    gh_authenticated(world, &login);
}

#[given(expr = "a configured account {string} with an invalid credential")]
fn configured_invalid_credential(world: &mut World, label: String) {
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token_env: Some("WORK_TOKEN".to_string()),
            ..AccountConfig::default()
        },
        Some(FakeCredential::Invalid),
    );
}

#[given(expr = "a configured account {string} with an environment credential")]
fn configured_environment_credential(world: &mut World, label: String) {
    world.add_account(
        &label,
        AccountConfig {
            auth: None,
            github_user: Some("octo-work".to_string()),
            token_env: Some("WORK_TOKEN".to_string()),
            ..AccountConfig::default()
        },
        Some(FakeCredential::Token {
            token: format!("token-TOKEN_{}", label.to_ascii_uppercase()),
        }),
    );
}

#[given(expr = "a configured account {string} with a command credential")]
fn configured_command_credential(world: &mut World, label: String) {
    world.add_account(
        &label,
        AccountConfig {
            auth: None,
            github_user: Some("octo-work".to_string()),
            token_command: Some("echo token".to_string()),
            ..AccountConfig::default()
        },
        Some(FakeCredential::Token {
            token: format!("token-TOKEN_{}", label.to_ascii_uppercase()),
        }),
    );
}

#[given(
    expr = "a configured account {string} with a personal access token in a file with mode {string}"
)]
fn configured_token_file_mode(world: &mut World, label: String, mode: String) {
    let mode_number: u32 = u32::from_str_radix(&mode, 8).unwrap_or(0o600);
    let path = world.add_private_token_file(mode_number, "token-TOKEN_WORK");
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token_file: Some(path.display().to_string()),
            ..AccountConfig::default()
        },
        None,
    );
}

#[given(
    expr = "a configured account {string} with a personal access token file that does not exist"
)]
fn configured_missing_token_file(world: &mut World, label: String) {
    let path = world.temp.path().join("missing.token");
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token_file: Some(path.display().to_string()),
            ..AccountConfig::default()
        },
        None,
    );
}

#[given(expr = "a configured account {string} with a personal access token in {string}")]
fn configured_token_file_path(world: &mut World, label: String, path: String) {
    if path.starts_with("~/") {
        let name = path.trim_start_matches("~/");
        let file = world.home.join(name);
        std::fs::write(&file, "token-TOKEN_WORK\n").expect("token file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("chmod");
        }
    }
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token_file: Some(path),
            ..AccountConfig::default()
        },
        None,
    );
}

#[given(
    expr = "a configured account {string} with an expected login {string} and a personal access token belonging to {string}"
)]
fn configured_expected_login_token(
    world: &mut World,
    label: String,
    login: String,
    belonging: String,
) {
    let token = format!("token-{label}-{belonging}");
    if let Ok(mut data) = world.data.lock() {
        data.tokens.insert(token.clone(), belonging);
    }
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some(login),
            token_env: Some(format!("{}_TOKEN", label.to_ascii_uppercase())),
            ..AccountConfig::default()
        },
        Some(FakeCredential::Token { token }),
    );
}

#[given(
    regex = r#"^a configured account "([^"]+)" with (no credential source|an unknown auth mode|two personal access token sources|gh OAuth but no expected login|gh OAuth and a personal access token source|a literal personal access token)$"#
)]
fn configured_invalid_matrix(world: &mut World, label: String, invalid: String) {
    let account = match invalid.as_str() {
        "no credential source" => AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            ..AccountConfig::default()
        },
        "gh OAuth but no expected login" => AccountConfig {
            auth: Some(AuthMode::GhOAuth),
            ..AccountConfig::default()
        },
        "gh OAuth and a personal access token source" => AccountConfig {
            auth: Some(AuthMode::GhOAuth),
            github_user: Some("octo-work".to_string()),
            token_env: Some("WORK_TOKEN".to_string()),
            ..AccountConfig::default()
        },
        "two personal access token sources" => AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token_env: Some("WORK_TOKEN".to_string()),
            token_command: Some("echo token".to_string()),
            ..AccountConfig::default()
        },
        "a literal personal access token" => AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some("octo-work".to_string()),
            token: Some("secret".to_string()),
            ..AccountConfig::default()
        },
        "an unknown auth mode" => {
            world.write_raw_config(&format!(
                "[accounts.{label}]\nauth = \"mystery\"\ngithub_user = \"octo-work\"\ntoken_env = \"WORK_TOKEN\"\n"
            ));
            return;
        }
        _ => unreachable!(),
    };
    world.add_account(&label, account, None);
}

#[given(
    regex = r#"^a configured account "([^"]+)" with (a relative token file path|a token file path that is a directory|an empty token file)$"#
)]
fn configured_unusable_token_file(world: &mut World, label: String, case: String) {
    let (path, expected_login) = match case.as_str() {
        "a relative token file path" => ("relative.token".to_string(), "octo-work".to_string()),
        "a token file path that is a directory" => {
            let path = world.temp.path().join("token-directory");
            std::fs::create_dir_all(&path).expect("token directory");
            (path.display().to_string(), "octo-work".to_string())
        }
        _ => {
            let path = world.add_private_token_file(0o600, "");
            (path.display().to_string(), "octo-work".to_string())
        }
    };
    world.add_account(
        &label,
        AccountConfig {
            auth: Some(AuthMode::GithubPat),
            github_user: Some(expected_login),
            token_file: Some(path),
            ..AccountConfig::default()
        },
        None,
    );
}

#[given(expr = "an invalid configuration")]
fn invalid_configuration(world: &mut World) {
    world.write_raw_config("no_accounts = true\n");
}

#[given(expr = "no configuration file exists")]
fn no_configuration_file(world: &mut World) {
    let _ = std::fs::remove_file(&world.paths.config);
    world.config = passuff_config::AppConfig::default();
    world.config.poll_interval_secs = 1;
    world.config_before = None;
}

#[given(expr = "a configuration file exists")]
fn configuration_exists(world: &mut World) {
    world.config = passuff_config::AppConfig::default();
    world.config.poll_interval_secs = 1;
    world.config.accounts.insert(
        "personal".to_string(),
        AccountConfig {
            auth: Some(AuthMode::GhOAuth),
            github_user: Some("octo-old".to_string()),
            ..AccountConfig::default()
        },
    );
    world.write_config();
    gh_authenticated(world, "octo-old");
    world.config_before = Some(world.config_text());
}

#[given(expr = "a world-readable token file")]
fn world_readable_token_file(world: &mut World) {
    world.add_world_readable_token_file();
}

#[given(expr = "the setup configuration directory cannot be written")]
fn setup_config_directory_cannot_be_written(_world: &mut World) {
    std::env::set_var("PASSUFF_TEST_FAIL_CONFIG_WRITE", "1");
}

#[given(expr = "the configuration directory cannot be written")]
fn config_directory_cannot_be_written(_world: &mut World) {
    std::env::set_var("PASSUFF_TEST_FAIL_CONFIG_WRITE", "1");
}

#[given(expr = "the state directory cannot be written")]
fn state_directory_cannot_be_written(_world: &mut World) {
    std::env::set_var("PASSUFF_TEST_FAIL_STATE_WRITE", "1");
}

#[given(expr = "the state file cannot be written")]
fn state_file_cannot_be_written(_world: &mut World) {
    std::env::set_var("PASSUFF_TEST_FAIL_STATE_WRITE", "1");
}

#[given(expr = "the state path is a directory")]
fn state_path_is_directory(world: &mut World) {
    let _ = std::fs::remove_file(&world.paths.state);
    std::fs::create_dir_all(&world.paths.state).expect("state directory");
}

#[given(expr = "no state file")]
fn no_state_file(world: &mut World) {
    let _ = std::fs::remove_file(&world.paths.state);
}

#[given(expr = "a corrupt persisted observation")]
fn corrupt_persisted_observation(world: &mut World) {
    world.write_raw_state("{ this is not json");
}

#[given(expr = "a version-1 state file with one watched pull request")]
fn version_one_state(world: &mut World) {
    world.write_version_one_state();
}

#[given(expr = "a state with {int} activity records")]
fn state_with_activity(world: &mut World, count: usize) {
    world.write_activity_state(count);
}

#[given(expr = "a state with one activity record")]
fn state_with_one_activity(world: &mut World) {
    world.write_activity_state(1);
}

#[given(expr = "the watch scope is limited to {string}")]
fn watch_scope_limited(world: &mut World, repository: String) {
    world.config.watch.mode = Some(WatchMode::Selected);
    world.config.watch.repositories = vec![repository];
    world.write_config();
}

#[given(expr = "the watch scope mode is {string}")]
fn given_watch_scope_mode(world: &mut World, mode: String) {
    world.config.watch.mode = Some(if mode == "selected" {
        WatchMode::Selected
    } else {
        WatchMode::All
    });
    world.write_config();
}

#[given(expr = "the watch scope contains {string}")]
fn given_watch_scope_contains(world: &mut World, repository: String) {
    if !world
        .config
        .watch
        .repositories
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(&repository))
    {
        world.config.watch.repositories.push(repository);
    }
    world.write_config();
}

#[given(expr = "the watch scope does not contain {string}")]
fn given_watch_scope_does_not_contain(world: &mut World, repository: String) {
    world
        .config
        .watch
        .repositories
        .retain(|candidate| !candidate.eq_ignore_ascii_case(&repository));
    world.write_config();
}

#[given(expr = "the focus relations are {string}")]
fn given_focus_relations(world: &mut World, relations: String) {
    let parsed = parse_relations(&relations);
    world.config.focus.relations = parsed;
    world.write_config();
}

#[given(expr = "the involvement relations are unknown")]
fn involvement_relations_unknown(world: &mut World) {
    world.seed_unknown_relations();
}

#[given(expr = "the reconciliation interval is {string} seconds")]
fn reconciliation_interval(world: &mut World, seconds: String) {
    world.config.reconciliation_interval_secs = seconds.parse().unwrap_or(1);
    world.write_config();
}

#[given(expr = "the sound is configured to {string}")]
fn sound_configured(world: &mut World, path: String) {
    let absolute = std::path::Path::new(&path);
    let absolute = if absolute.is_absolute() {
        absolute.to_path_buf()
    } else {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(absolute)
            .canonicalize()
            .unwrap_or_else(|_| absolute.to_path_buf())
    };
    world.sound_display = Some(path);
    world.config.notifications.sound_file = absolute.display().to_string();
    world.write_config();
}

#[given(expr = "sound is disabled in configuration")]
fn sound_disabled(world: &mut World) {
    world.config.notifications.sound = false;
    world.write_config();
}

#[given(expr = "the desktop notification is disabled in configuration")]
fn desktop_disabled(world: &mut World) {
    world.config.notifications.desktop = false;
    world.write_config();
}

#[given(expr = "bot comments are {string} in configuration")]
fn bot_comments(world: &mut World, value: String) {
    world.config.events.bot_comments = value == "enabled";
    world.write_config();
}

#[given(expr = "the {string} event family is disabled in configuration")]
fn event_family_disabled(world: &mut World, family: String) {
    match family.as_str() {
        "comments" => world.config.events.comments = false,
        "reviews" => world.config.events.reviews = false,
        "ci_failure" => world.config.events.ci_failure = false,
        "ci_recovery" => world.config.events.ci_recovery = false,
        "state_changes" => world.config.events.state_changes = false,
        _ => {}
    }
    world.write_config();
}

#[given(expr = "the desktop notification surface is unavailable")]
fn desktop_unavailable(world: &mut World) {
    world.delivery.set_available(false);
}

#[given(expr = "the sound player is unavailable")]
fn sound_unavailable(world: &mut World) {
    world.sound.set_available(false);
}

#[given(expr = "the terminal is interactive")]
fn terminal_interactive(world: &mut World) {
    world.interactive = true;
}

#[given(expr = "the terminal is not interactive")]
fn terminal_not_interactive(world: &mut World) {
    world.interactive = false;
}

#[given(expr = "gh is authenticated as {string}")]
fn gh_authenticated_as(world: &mut World, login: String) {
    if let Ok(mut data) = world.data.lock() {
        data.gh_authenticated.insert(login);
    }
}

#[given(expr = "gh is not authenticated as {string}")]
fn gh_not_authenticated_as(world: &mut World, login: String) {
    if let Ok(mut data) = world.data.lock() {
        data.gh_authenticated.remove(&login);
    }
}

#[given(expr = "gh login always fails")]
fn gh_login_always_fails(world: &mut World) {
    if let Ok(mut data) = world.data.lock() {
        data.gh_login_fails = true;
    }
}

#[given(
    expr = "the account has {string} API points left until the allowance resets in {string} seconds"
)]
fn account_allowance(world: &mut World, remaining: String, reset: String) {
    world.set_allowance(
        remaining.parse().unwrap_or(5000),
        reset.parse().unwrap_or(3600),
    );
}

#[given(expr = "an authored pull request {string} by {string}")]
fn authored_pull_request(world: &mut World, key_value: String, author: String) {
    world.add_pull_request(&key(&key_value), &author);
}

#[given(expr = "a pull request {string} by {string}")]
fn pull_request(world: &mut World, key_value: String, author: String) {
    world.add_pull_request(&key(&key_value), &author);
}

#[given(expr = "{int} authored pull requests in {string} by {string}")]
fn many_authored_pull_requests(world: &mut World, count: u64, repository: String, author: String) {
    for number in 1..=count {
        let key_value = format!("{repository}#{number}");
        world.add_pull_request(&key(&key_value), &author);
    }
}

#[given(expr = "the account has no authored pull requests")]
fn account_has_no_pull_requests(_world: &mut World) {}

#[given(expr = "the watcher has already observed {string}")]
fn watcher_observed(world: &mut World, key_value: String) {
    world.seed_observed(&[key(&key_value)], false);
}

#[given(expr = "the watcher has already observed {string} with a failing CI result")]
fn watcher_observed_failing(world: &mut World, key_value: String) {
    world.seed_observed(&[key(&key_value)], true);
}

#[given(expr = "the watcher has already observed {string} and {string}")]
fn watcher_observed_two(world: &mut World, first: String, second: String) {
    world.seed_observed(&[key(&first), key(&second)], false);
}

#[given(expr = "the watcher has already observed all of them")]
fn watcher_observed_all(world: &mut World) {
    let keys: Vec<PullRequestKey> = world
        .all_pull_requests()
        .into_iter()
        .map(|snapshot| snapshot.key)
        .collect();
    world.seed_observed(&keys, false);
}

#[given(expr = "a state already watches {string}")]
fn state_already_watches(world: &mut World, key_value: String) {
    world.seed_observed(&[key(&key_value)], false);
}

#[given(expr = "the pull request already has a comment by {string}")]
fn pull_request_has_comment(world: &mut World, author: String) {
    let key_value = world
        .all_pull_requests()
        .first()
        .map(|snapshot| snapshot.key.clone());
    if let Some(key_value) = key_value {
        let now = world.clock.now();
        world.add_comment(
            &key_value,
            &author,
            "Existing comment",
            Some(now - Duration::hours(2)),
        );
    }
}

#[given(expr = "the pull request {string} is {string}")]
fn pull_request_is_state(world: &mut World, key_value: String, state: String) {
    world.set_state(&key(&key_value), pr_state(&state));
}

#[given(expr = "the pull request {string} is assigned to me")]
fn pull_request_assigned_to_me(world: &mut World, key_value: String) {
    add_relation(world, &key_value, FocusRelation::Assigned);
}

#[given(expr = "the pull request {string} is assigned to {string}")]
fn pull_request_assigned_to(world: &mut World, key_value: String, assignee: String) {
    let pull_request_key = key(&key_value);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.assignees = vec![assignee];
        }
    }
}

#[given(expr = "a team review is requested on {string}")]
fn team_review_requested(world: &mut World, key_value: String) {
    add_relation(world, &key_value, FocusRelation::TeamReviewRequested);
}

#[given(expr = "the pull request {string} is ignored")]
fn pull_request_ignored(world: &mut World, key_value: String) {
    if !world
        .config
        .watch
        .ignored_pull_requests
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(&key_value))
    {
        world.config.watch.ignored_pull_requests.push(key_value);
    }
    world.write_config();
}

#[given(expr = "the pull request {string} has the description {string}")]
fn pull_request_description(world: &mut World, key_value: String, description: String) {
    set_pr_body(world, &key_value, description);
}

#[given(expr = "the pull request {string} has a multi-line description")]
fn pull_request_multiline_description(world: &mut World, key_value: String) {
    set_pr_body(
        world,
        &key_value,
        "Summary\n\nFirst paragraph\n\nSecond paragraph".to_string(),
    );
}

#[given(expr = "the pull request {string} has a markdown description")]
fn pull_request_markdown_description(world: &mut World, key_value: String) {
    set_pr_body(
        world,
        &key_value,
        "## Summary\n\n- first item\n\n**bold** text".to_string(),
    );
}

#[given(expr = "the pull request {string} has a long description")]
fn pull_request_long_description(world: &mut World, key_value: String) {
    let body = (1..=40)
        .map(|number| format!("line {number}"))
        .collect::<Vec<_>>()
        .join("\n");
    set_pr_body(world, &key_value, body);
}

#[when(
    expr = "GitHub reports stack {int} for {string} at position {int} of {int} with base {string}"
)]
fn github_reports_stack_when(
    world: &mut World,
    number: u64,
    key_value: String,
    position: u64,
    size: u64,
    base: String,
) {
    github_reports_stack(world, number, key_value, position, size, base);
}

#[given(
    expr = "GitHub reports stack {int} for {string} at position {int} of {int} with base {string}"
)]
fn github_reports_stack(
    world: &mut World,
    number: u64,
    key_value: String,
    position: u64,
    size: u64,
    base: String,
) {
    let pull_request_key = key(&key_value);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.stack = Some(passuff_core::StackInfo {
                number,
                size,
                position,
                base,
            });
        }
    }
}

#[given(expr = "the configuration file is corrupted")]
fn configuration_file_corrupted(world: &mut World) {
    world.write_raw_config("this is not = valid toml");
    world.config_corrupted = true;
}

#[when(expr = "the configuration file is corrupted")]
fn configuration_file_corrupted_when(world: &mut World) {
    world.write_raw_config("this is not = valid toml");
    world.config_corrupted = true;
}

#[given(expr = "a watcher is running with a configuration path")]
fn watcher_running(world: &mut World) {
    world.watcher_running = true;
}

#[when(expr = "an authored pull request {string} by {string} is created after the last check")]
fn authored_created_after_last_check(world: &mut World, key_value: String, author: String) {
    world.add_pull_request(&key(&key_value), &author);
    let now = world.clock.now();
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&key(&key_value)) {
            pull_request.created_at = Some(now + Duration::seconds(60));
            pull_request.updated_at = Some(now + Duration::seconds(60));
        }
    }
}

#[when(
    expr = "an authored pull request {string} by {string} appears with an existing comment by {string}"
)]
fn authored_appears_with_comment(
    world: &mut World,
    key_value: String,
    author: String,
    commenter: String,
) {
    world.add_pull_request(&key(&key_value), &author);
    let now = world.clock.now();
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&key(&key_value)) {
            pull_request.created_at = Some(now - Duration::hours(3));
            pull_request.updated_at = Some(now - Duration::hours(3));
        }
    }
    world.add_comment(
        &key(&key_value),
        &commenter,
        "Existing comment",
        Some(now - Duration::hours(2)),
    );
}

#[when(
    expr = "an authored pull request {string} by {string} is created and {string} between checks"
)]
fn authored_short_lived(world: &mut World, key_value: String, author: String, state: String) {
    world.add_pull_request(&key(&key_value), &author);
    let now = world.clock.now();
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&key(&key_value)) {
            pull_request.created_at = Some(now + Duration::seconds(60));
            pull_request.updated_at = Some(now + Duration::seconds(120));
            pull_request.state = pr_state(&state);
            if pull_request.state == PullRequestState::Merged {
                pull_request.merged_at = Some(now + Duration::seconds(120));
            } else {
                pull_request.closed_at = Some(now + Duration::seconds(120));
            }
        }
    }
}

#[when(expr = "an authored pull request {string} by {string} appears")]
fn authored_appears(world: &mut World, key_value: String, author: String) {
    world.add_pull_request(&key(&key_value), &author);
    let now = world.clock.now();
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&key(&key_value)) {
            pull_request.created_at = Some(now - Duration::hours(3));
            pull_request.updated_at = Some(now - Duration::hours(3));
        }
    }
}

#[when(expr = "the pull request {string} becomes {string}")]
fn pull_request_becomes(world: &mut World, key_value: String, state: String) {
    world.set_state(&key(&key_value), pr_state(&state));
}

#[when(expr = "the pull request {string} becomes {string} while the watcher is stopped")]
fn pull_request_becomes_while_stopped(world: &mut World, key_value: String, state: String) {
    world.set_state(&key(&key_value), pr_state(&state));
}

#[when(expr = "the pull request {string} is reopened")]
fn pull_request_reopened(world: &mut World, key_value: String) {
    let pull_request_key = key(&key_value);
    world.set_state(&pull_request_key, PullRequestState::Open);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.merged_at = None;
            pull_request.closed_at = None;
        }
    }
}

#[when(expr = "the configured account {string} merges {string}")]
fn configured_account_merges(world: &mut World, _account: String, key_value: String) {
    world.set_state(&key(&key_value), PullRequestState::Merged);
}

#[when(expr = "the pull request {string} receives a long comment from {string}")]
fn long_comment(world: &mut World, key_value: String, author: String) {
    let body = (1..=40)
        .map(|number| format!("comment line {number}"))
        .collect::<Vec<_>>()
        .join("\n");
    world.add_comment(&key(&key_value), &author, &body, None);
}

#[when(expr = "{string} comments {string} on {string}")]
fn comment(world: &mut World, author: String, body: String, key_value: String) {
    let author = author.trim_start_matches('@').to_string();
    world.add_comment(&key(&key_value), &author, &body, None);
}

#[when(expr = "{string} comments {string} on {string} at {string}")]
fn comment_at(world: &mut World, author: String, body: String, key_value: String, time: String) {
    let author = author.trim_start_matches('@').to_string();
    world.add_comment(&key(&key_value), &author, &body, Some(parse_time(&time)));
}

#[when(expr = "{string} comments {string} on {string} after the first observation")]
fn comment_after_first_observation(
    world: &mut World,
    author: String,
    body: String,
    key_value: String,
) {
    let author = author.trim_start_matches('@').to_string();
    world.add_comment(&key(&key_value), &author, &body, None);
}

#[given(expr = "{string} comments {string} on {string} after the first observation")]
fn comment_after_first_observation_given(
    world: &mut World,
    author: String,
    body: String,
    key_value: String,
) {
    let author = author.trim_start_matches('@').to_string();
    world.add_comment(&key(&key_value), &author, &body, None);
}

#[when(expr = "{string} writes a review comment {string} on {string}")]
fn review_comment(world: &mut World, author: String, body: String, key_value: String) {
    let author = author.trim_start_matches('@').to_string();
    world.add_review_comment(&key(&key_value), &author, &body);
}

#[when(expr = "{string} submits a {string} review on {string}")]
fn submit_review(world: &mut World, author: String, state: String, key_value: String) {
    let author = author.trim_start_matches('@').to_string();
    world.add_review(
        &key(&key_value),
        &author,
        review_state(&state),
        "Review body",
    );
}

#[when(expr = "{string} submits an {string} review on {string}")]
fn submit_review_an(world: &mut World, author: String, state: String, key_value: String) {
    let author = author.trim_start_matches('@').to_string();
    world.add_review(
        &key(&key_value),
        &author,
        review_state(&state),
        "Review body",
    );
}

#[given(expr = "no prior activity exists for {string}")]
fn no_prior_activity(_world: &mut World, _key_value: String) {}

#[given(expr = "the CI result is {string} for {string}")]
fn ci_result_is_for(world: &mut World, state: String, key_value: String) {
    world.set_ci(&key(&key_value), ci_result(&state));
}

#[given(expr = "the pull request is {string} for {string}")]
fn pull_request_is_for(world: &mut World, state: String, key_value: String) {
    world.set_state(&key(&key_value), pr_state(&state));
}

#[when(expr = "the CI result becomes {string} on {string}")]
fn ci_becomes_on(world: &mut World, state: String, key_value: String) {
    world.set_ci(&key(&key_value), ci_result(&state));
}

#[when(expr = "the pull request becomes {string} on {string}")]
fn pull_request_becomes_on(world: &mut World, state: String, key_value: String) {
    world.set_state(&key(&key_value), pr_state(&state));
}

#[when(expr = "{string} pushes a commit to {string}")]
fn push_commit(world: &mut World, _author: String, key_value: String) {
    let pull_request_key = key(&key_value);
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.head_oid = Some(format!(
                "{}-pushed",
                pull_request.head_oid.clone().unwrap_or_default()
            ));
        }
    }
}

#[when(expr = "the CI result on {string} becomes {string}")]
fn ci_becomes(world: &mut World, key_value: String, state: String) {
    world.set_ci(&key(&key_value), ci_result(&state));
}

#[when(expr = "the CI result on {string} is {string}")]
fn ci_is(world: &mut World, key_value: String, state: String) {
    world.set_ci(&key(&key_value), ci_result(&state));
}

#[given(expr = "the CI result on {string} is {string}")]
fn ci_is_given(world: &mut World, key_value: String, state: String) {
    world.set_ci(&key(&key_value), ci_result(&state));
}

#[when(regex = r#"^a "(label|draft|assignee|milestone|base branch)" change occurs on "([^"]+)"$"#)]
fn metadata_change(world: &mut World, _change: String, key_value: String) {
    let pull_request_key = key(&key_value);
    let now = world.clock.now();
    if let Ok(mut data) = world.data.lock() {
        if let Some(pull_request) = data.pull_requests.get_mut(&pull_request_key) {
            pull_request.updated_at = Some(now + Duration::seconds(7));
        }
    }
}

#[when(expr = "the next observation fails")]
fn next_observation_fails(world: &mut World) {
    if let Ok(mut data) = world.data.lock() {
        data.fail_next_observation = true;
    }
}

#[when(expr = "GitHub becomes unavailable for the next observation")]
fn github_unavailable(world: &mut World) {
    if let Ok(mut data) = world.data.lock() {
        data.unavailable_next = true;
    }
}

#[when(expr = "the credential for {string} becomes a token belonging to {string}")]
fn credential_becomes(world: &mut World, label: String, login: String) {
    let token = format!("token-other-{login}");
    if let Ok(mut data) = world.data.lock() {
        data.credentials.insert(
            label,
            FakeCredential::Token {
                token: token.clone(),
            },
        );
        data.tokens.insert(token, login);
    }
}

#[when(expr = "the browser will fail to open")]
fn browser_will_fail(world: &mut World) {
    world.browser.set_fail(true);
}

#[then(expr = "the transcript reports no notifications")]
fn transcript_no_notifications(world: &mut World) {
    let transcript = world.latest();
    assert!(
        transcript.contains("no notifications"),
        "expected no notifications, got:\n{transcript}"
    );
    assert!(
        !transcript.contains("notification for"),
        "expected no notification lines, got:\n{transcript}"
    );
}

#[then(expr = "the transcript reports Merged")]
fn transcript_merged(world: &mut World) {
    assert!(
        world.latest().contains("Merged"),
        "got:\n{}",
        world.latest()
    );
}

#[then(expr = "the transcript reports Closed")]
fn transcript_closed(world: &mut World) {
    assert!(
        world.latest().contains("Closed"),
        "got:\n{}",
        world.latest()
    );
}

fn parse_relations(value: &str) -> Vec<FocusRelation> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter_map(FocusRelation::parse)
        .collect()
}

#[allow(dead_code)]
fn _unused(_: BTreeMap<String, String>, _: CiResult, _: ReviewState) {}
