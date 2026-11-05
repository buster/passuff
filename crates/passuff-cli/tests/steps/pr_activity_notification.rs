use passuff_config::SoundFile;
use passuff_core::{CiResult, CiState, Comment, PrState, Review, ReviewComment};
use regex::Regex;

use cucumber::{given, then, when};

use crate::support::{ci_failing, ci_success, AccountAuth, World};

#[given(regex = r#"^a configured account "([^"]+)" with login "([^"]+)"$"#)]
async fn configured_account(world: &mut World, label: String, login: String) {
    world.add_account(&label, &login, false);
    if world.e2e {
        world.e2e_configure_account(&label, &login).await;
    }
}

#[given(regex = r#"^a configured account "([^"]+)" with an invalid credential$"#)]
async fn invalid_account(world: &mut World, label: String) {
    world.add_account(&label, "invalid", true);
}

#[given(regex = r#"^an authored pull request "([^"]+)" by "([^"]+)"$"#)]
async fn authored_pull_request(world: &mut World, key: String, author: String) {
    world.add_pr(&key, &author);
    if world.e2e {
        world.e2e_add_pull_request(&key, &author).await;
    }
}

#[given(regex = r#"^a pull request "([^"]+)" by "([^"]+)"$"#)]
async fn pull_request(world: &mut World, key: String, author: String) {
    world.add_pr(&key, &author);
    if world.e2e {
        world.e2e_add_pull_request(&key, &author).await;
    }
}

#[given(regex = r#"^the watcher has already observed "([^"]+)"$"#)]
async fn already_observed(world: &mut World, _key: String) {
    world.pending_baseline = true;
    if world.e2e {
        world.e2e_pending_baseline = true;
    }
    world.ensure_baseline().await;
}

#[given(regex = r#"^the watcher has already observed "([^"]+)" with a failing CI result$"#)]
async fn already_observed_failing(world: &mut World, key: String) {
    world.pr_mut(&key).ci = ci_failing();
    world.pending_baseline = true;
    if world.e2e {
        world.e2e_pending_baseline = true;
    }
}

#[given(regex = r#"^the watcher has already observed "([^"]+)" and "([^"]+)"$"#)]
async fn already_observed_two(world: &mut World, _a: String, _b: String) {
    world.pending_baseline = true;
    if world.e2e {
        world.e2e_pending_baseline = true;
    }
}

#[given(regex = r#"^the pull request "([^"]+)" has the description "(.*)"$"#)]
async fn pull_request_description(world: &mut World, key: String, description: String) {
    world.pr_mut(&key).description = description;
}

#[given(regex = r#"^the pull request "([^"]+)" has a multi-line description$"#)]
async fn pull_request_multiline_description(world: &mut World, key: String) {
    world.pr_mut(&key).description = "Summary\nFirst paragraph\n\nSecond paragraph".to_string();
}

#[given(regex = r#"^the pull request "([^"]+)" has a markdown description$"#)]
async fn pull_request_markdown_description(world: &mut World, key: String) {
    world.pr_mut(&key).description =
        "## Summary\n\n- first item\n- second item\n\n**bold** text".to_string();
}

#[given(regex = r#"^the pull request "([^"]+)" has a long description$"#)]
async fn pull_request_long_description(world: &mut World, key: String) {
    world.pr_mut(&key).description = (1..=40)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
}

#[given(regex = r#"^the pull request "([^"]+)" is assigned to "([^"]+)"$"#)]
async fn pull_request_assignee(world: &mut World, key: String, login: String) {
    world.pr_mut(&key).assignees.push(login);
}

#[given(regex = r#"^the pull request "([^"]+)" is assigned to me$"#)]
async fn assigned_pull_request(world: &mut World, key: String) {
    world
        .involved_pull_requests
        .lock()
        .expect("involved")
        .entry(key)
        .or_default()
        .push(passuff_config::Involvement::Assigned);
}

#[given(regex = r#"^a team review is requested on "([^"]+)"$"#)]
async fn team_review_requested(world: &mut World, key: String) {
    world
        .involved_pull_requests
        .lock()
        .expect("involved")
        .entry(key)
        .or_default()
        .push(passuff_config::Involvement::TeamReviewRequested);
}

#[given(regex = r#"^the pull request "([^"]+)" is ignored$"#)]
async fn ignored_pull_request(world: &mut World, key: String) {
    world.ignored_pull_requests.push(key);
}

#[given(regex = r#"^the pull request already has a comment by "([^"]+)"$"#)]
async fn existing_comment(world: &mut World, author: String) {
    let key = world
        .pull_requests
        .keys()
        .next()
        .cloned()
        .expect("pull request");
    world.pr_mut(&key).comments.push(Comment {
        id: format!("existing-{author}"),
        author,
        body: "Existing comment".to_string(),
        created_at: "2026-11-02T10:00:00Z".to_string(),
    });
}

#[given(regex = r#"^a corrupt persisted observation$"#)]
async fn corrupt_state(world: &mut World) {
    std::fs::write(world.state_path(), "{ not json").expect("corrupt state");
}

#[given(regex = r#"^an invalid configuration$"#)]
async fn invalid_configuration(world: &mut World) {
    world.invalid_config = true;
}

#[given(regex = r#"^the state file cannot be written$"#)]
async fn failing_state(world: &mut World) {
    world.failing_store = true;
}

#[given(regex = r#"^the desktop notification surface is unavailable$"#)]
async fn unavailable_desktop(world: &mut World) {
    world.failing_delivery = true;
}

#[given(regex = r#"^the sound player is unavailable$"#)]
async fn unavailable_sound(world: &mut World) {
    world.unavailable_sound = true;
}

#[given(regex = r#"^the desktop notification is disabled in configuration$"#)]
async fn desktop_disabled(world: &mut World) {
    world.notifications.desktop = false;
}

#[given(regex = r#"^sound is disabled in configuration$"#)]
async fn sound_disabled(world: &mut World) {
    world.notifications.sound = false;
}

#[given(regex = r#"^the sound is configured to "([^"]+)"$"#)]
async fn sound_configured(world: &mut World, path: String) {
    world.notifications.sound_file = SoundFile::Path(path.into());
}

#[given(regex = r#"^bot comments are "([^"]+)" in configuration$"#)]
async fn bot_comments(world: &mut World, value: String) {
    world.events.bot_comments = value == "enabled";
}

#[given(regex = r#"^the "([^"]+)" event family is disabled in configuration$"#)]
async fn event_family_disabled(world: &mut World, family: String) {
    match family.as_str() {
        "comments" => world.events.comments = false,
        "reviews" => world.events.reviews = false,
        "ci_failure" => world.events.ci_failure = false,
        "ci_recovery" => world.events.ci_recovery = false,
        "state_changes" => world.events.state_changes = false,
        other => panic!("unknown event family {other}"),
    }
}

#[given(regex = r#"^(.+) for "([^"]+)"$"#)]
async fn family_setup(world: &mut World, setup: String, key: String) {
    if setup == "no prior activity exists" {
        return;
    }
    let pr = world.pr_mut(&key);
    if setup == r#"the CI result is "successful""# {
        pr.ci = ci_success();
    } else if setup == r#"the CI result is "failing""# {
        pr.ci = ci_failing();
    } else if setup == r#"the pull request is "open""# {
        pr.state = PrState::Open;
        pr.merged = false;
    } else {
        panic!("unknown family setup {setup}");
    }
}

#[given(regex = r#"^the account has no authored pull requests$"#)]
async fn no_authored_pull_requests(world: &mut World) {
    world.pull_requests.clear();
    if world.e2e {
        world.e2e_clear_pull_requests().await;
    }
}

#[when(regex = r#"^(.+) on "([^"]+)"$"#)]
async fn event_on_pull_request(world: &mut World, action: String, key: String) {
    world.ensure_baseline().await;
    apply_action(world, &action, &key).await;
}

#[when(regex = r#"^"([^"]+)" pushes a commit to "([^"]+)"$"#)]
async fn push_commit(world: &mut World, _author: String, key: String) {
    world.ensure_baseline().await;
    world.pr_mut(&key).head_oid = "def456".to_string();
}

#[when(regex = r#"^the CI result on "([^"]+)" becomes "([^"]+)"$"#)]
async fn ci_result_on(world: &mut World, key: String, state: String) {
    world.ensure_baseline().await;
    world.pr_mut(&key).ci = ci_for(&state);
}

#[when(regex = r#"^the pull request "([^"]+)" becomes "([^"]+)" while the watcher is stopped$"#)]
async fn state_while_stopped(world: &mut World, key: String, state: String) {
    world.ensure_baseline().await;
    set_state(world, &key, &state);
}

#[given(regex = r#"^the pull request "([^"]+)" is "(merged|closed|open)"$"#)]
async fn pull_request_state(world: &mut World, key: String, state: String) {
    set_state(world, &key, &state);
}

#[when(regex = r#"^the pull request "([^"]+)" becomes "([^"]+)"$"#)]
async fn state_change(world: &mut World, key: String, state: String) {
    world.ensure_baseline().await;
    set_state(world, &key, &state);
}

#[when(regex = r#"^the pull request "([^"]+)" is reopened$"#)]
async fn reopened(world: &mut World, key: String) {
    world.ensure_baseline().await;
    let pr = world.pr_mut(&key);
    pr.state = PrState::Open;
    pr.merged = false;
}

#[when(regex = r#"^the configured account "([^"]+)" merges "([^"]+)"$"#)]
async fn own_merge(world: &mut World, _login: String, key: String) {
    world.ensure_baseline().await;
    set_state(world, &key, "merged");
}

#[when(
    regex = r#"^an authored pull request "([^"]+)" by "([^"]+)" appears with an existing comment by "([^"]+)"$"#
)]
async fn first_seen_later(world: &mut World, key: String, author: String, commenter: String) {
    world.ensure_baseline().await;
    let pr = world.add_pr(&key, &author);
    pr.comments.push(Comment {
        id: "existing-later".to_string(),
        author: commenter.clone(),
        body: "Existing comment".to_string(),
        created_at: "2026-11-02T11:30:00Z".to_string(),
    });
    if world.e2e {
        world.e2e_add_pull_request(&key, &author).await;
        world
            .e2e_set_comment(&key, "existing-later", &commenter, "Existing comment")
            .await;
    }
}

#[when(
    regex = r#"^an authored pull request "([^"]+)" by "([^"]+)" is created and "([^"]+)" between checks$"#
)]
async fn short_lived(world: &mut World, key: String, author: String, state: String) {
    world.ensure_baseline().await;
    let pr = world.add_pr(&key, &author);
    pr.created_at = "2026-11-02T12:00:02Z".to_string();
    pr.updated_at = "2026-11-02T12:00:02Z".to_string();
    pr.state = if state == "merged" {
        PrState::Merged
    } else {
        PrState::Closed
    };
    pr.merged = state == "merged";
    if world.e2e {
        world.e2e_add_pull_request(&key, &author).await;
    }
}

#[when(
    regex = r#"^an authored pull request "([^"]+)" by "([^"]+)" is created after the last check$"#
)]
async fn created_after_last_check(world: &mut World, key: String, author: String) {
    let pull_request = world.add_pr(&key, &author);
    pull_request.created_at = "2026-11-02T12:00:02Z".to_string();
    pull_request.updated_at = "2026-11-02T12:00:02Z".to_string();
}

#[when(regex = r#"^passuff runs one check$"#)]
async fn runs_one_check(world: &mut World) {
    if world.e2e {
        world.e2e_run_check().await;
    } else {
        world.ensure_baseline().await;
        world.run_check_internal().await;
    }
}

#[when(regex = r#"^passuff restarts and runs one check$"#)]
async fn restarts_and_runs_one_check(world: &mut World) {
    if world.e2e {
        world.e2e_run_check().await;
    } else {
        world.run_check_internal().await;
    }
}

#[then(regex = r#"^the transcript reports "([^"]+)"$"#)]
async fn transcript_reports(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected transcript to contain {expected:?}, got {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the latest transcript reports "([^"]+)"$"#)]
async fn latest_transcript_reports(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected latest transcript to contain {expected:?}, got {:?}",
        world.last_transcript
    );
}

#[then(regex = r#"^the transcript reports no notifications$"#)]
async fn no_notifications(world: &mut World) {
    assert!(
        world.last_report().notifications.is_empty(),
        "expected no notifications, got {:?}",
        world.last_report().notifications
    );
}

#[then(regex = r#"^the transcript reports a notification for "([^"]+)"$"#)]
async fn reports_notification(world: &mut World, key: String) {
    let report = world
        .last_report
        .as_ref()
        .unwrap_or_else(|| panic!("run a check first; transcript: {:?}", world.last_transcript));
    assert!(
        report
            .notifications
            .iter()
            .any(|notification| notification.key.to_string() == key),
        "expected a notification for {key}, got {:?}; transcript: {:?}",
        report.notifications,
        world.last_transcript
    );
}

#[then(regex = r#"^the transcript reports one notification for "([^"]+)"$"#)]
async fn reports_one_notification(world: &mut World, key: String) {
    let count = world
        .last_report()
        .notifications
        .iter()
        .filter(|notification| notification.key.to_string() == key)
        .count();
    assert_eq!(count, 1, "expected one notification for {key}");
}

#[then(regex = r#"^the desktop notification reports "([^"]+)"$"#)]
async fn desktop_notification_reports(world: &mut World, expected: String) {
    let notification = world
        .e2e_last_notification()
        .or_else(|| world.last_notification())
        .expect("desktop notification");
    let body = notification
        .get("body")
        .and_then(|value| value.as_str())
        .or_else(|| notification.get("summary").and_then(|value| value.as_str()))
        .unwrap_or_default();
    assert!(
        body.contains(&expected),
        "expected desktop body to contain {expected:?}, got {body:?}"
    );
}

#[then(regex = r#"^the desktop notification is recorded$"#)]
async fn desktop_notification_recorded(world: &mut World) {
    assert!(
        world.e2e_last_notification().is_some() || world.last_notification().is_some(),
        "expected a desktop notification"
    );
}

#[then(regex = r#"^no desktop notification is recorded$"#)]
async fn no_desktop_notification(world: &mut World) {
    assert!(
        world.e2e_last_notification().is_none() && world.last_notification().is_none(),
        "expected no desktop notification"
    );
}

#[then(regex = r#"^the configured sound is played once$"#)]
async fn sound_played_once(world: &mut World) {
    let calls = world.sound_calls();
    assert_eq!(calls.len(), 1, "expected one sound, got {calls:?}");
}

#[then(regex = r#"^the configured sound file "([^"]+)" is played once$"#)]
async fn sound_file_played_once(world: &mut World, expected: String) {
    let calls = world.sound_calls();
    assert_eq!(calls, vec![expected], "expected the configured sound file");
}

#[then(regex = r#"^no sound is played$"#)]
async fn no_sound(world: &mut World) {
    assert!(
        world.sound_calls().is_empty(),
        "expected no sound, got {:?}",
        world.sound_calls()
    );
}

#[then(regex = r#"^the notification mentions "([^"]+)", "([^"]+)", and "([^"]+)"$"#)]
async fn notification_mentions(world: &mut World, first: String, second: String, third: String) {
    let body = world
        .e2e_last_notification()
        .or_else(|| world.last_notification())
        .and_then(|value| value.get("body").cloned())
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    for expected in [first, second, third] {
        assert!(
            body.contains(&expected),
            "expected body to contain {expected:?}, got {body:?}"
        );
    }
}

#[then(regex = r#"^the exit status is (.+)$"#)]
async fn exit_status(world: &mut World, expected: String) {
    if expected == "nonzero" {
        assert_ne!(world.last_status, 0, "expected a nonzero exit status");
    } else {
        let expected: i32 = expected.parse().expect("numeric exit status");
        assert_eq!(world.last_status, expected);
    }
}

#[then(regex = r#"^the report records a baseline for "([^"]+)"$"#)]
async fn report_baseline(world: &mut World, key: String) {
    assert!(
        world
            .last_report()
            .baselines_recorded
            .iter()
            .any(|baseline| baseline.to_string() == key),
        "expected a baseline for {key}"
    );
}

async fn apply_action(world: &mut World, action: &str, key: &str) {
    if let Some(caps) = capture(r#"^"([^"]+)" comments "(.*)"$"#, action) {
        let author = caps[0].clone();
        let body = caps[1].clone();
        let comment = Comment {
            id: format!("comment-{}", world.pr_mut(key).comments.len() + 1),
            author: author.clone(),
            body,
            created_at: "2026-11-02T12:00:01Z".to_string(),
        };
        world.pr_mut(key).comments.push(comment.clone());
        world.pr_mut(key).updated_at = comment.created_at.clone();
        if world.e2e {
            world
                .e2e_set_comment(key, &comment.id, &comment.author, &comment.body)
                .await;
        }
        return;
    }
    if let Some(caps) = capture(r#"^"([^"]+)" writes a review comment "(.*)"$"#, action) {
        let author = caps[0].clone();
        let body = caps[1].clone();
        let comment = ReviewComment {
            id: format!(
                "review-comment-{}",
                world.pr_mut(key).review_comments.len() + 1
            ),
            author,
            body,
            created_at: "2026-11-02T12:00:01Z".to_string(),
        };
        world.pr_mut(key).review_comments.push(comment.clone());
        if world.e2e {
            world
                .e2e_set_review_comment(key, &comment.id, &comment.author, &comment.body)
                .await;
        }
        return;
    }
    if let Some(caps) = capture(r#"^"([^"]+)" submits an? "(.*)" review$"#, action) {
        let author = caps[0].clone();
        let state = caps[1].clone();
        let review = Review {
            id: format!("review-{}", world.pr_mut(key).reviews.len() + 1),
            author,
            state,
            body: String::new(),
            submitted_at: "2026-11-02T12:00:01Z".to_string(),
        };
        world.pr_mut(key).reviews.push(review.clone());
        if world.e2e {
            world
                .e2e_set_review(key, &review.id, &review.author, &review.state)
                .await;
        }
        return;
    }
    if let Some(caps) = capture(r#"^the CI result becomes "(.*)"$"#, action) {
        let state = caps[0].clone();
        world.pr_mut(key).ci = ci_for(&state);
        if world.e2e {
            world.e2e_set_ci(key, &state).await;
        }
        return;
    }
    if let Some(caps) = capture(r#"^the pull request becomes "(.*)"$"#, action) {
        let state = caps[0].clone();
        set_state(world, key, &state);
        if world.e2e {
            world.e2e_set_state(key, &state).await;
        }
        return;
    }
    if let Some(caps) = capture(r#"^a "(.*)" change occurs$"#, action) {
        let change = caps[0].clone();
        let pr = world.pr_mut(key);
        match change.as_str() {
            "label" => pr.labels.push("changed".to_string()),
            "draft" => pr.is_draft = !pr.is_draft,
            "assignee" => pr.assignees.push("someone".to_string()),
            "milestone" => pr.milestone = Some("next".to_string()),
            "base branch" => pr.base_ref = "develop".to_string(),
            other => panic!("unknown metadata change {other}"),
        }
        return;
    }
    if action == "the pull request is reopened" {
        let pr = world.pr_mut(key);
        pr.state = PrState::Open;
        pr.merged = false;
        if world.e2e {
            world.e2e_set_state(key, "open").await;
        }
        return;
    }
    panic!("unhandled action {action:?}");
}

fn set_state(world: &mut World, key: &str, state: &str) {
    let pr = world.pr_mut(key);
    match state {
        "merged" => {
            pr.state = PrState::Merged;
            pr.merged = true;
        }
        "closed" => {
            pr.state = PrState::Closed;
            pr.merged = false;
        }
        "open" => {
            pr.state = PrState::Open;
            pr.merged = false;
        }
        other => panic!("unknown PR state {other}"),
    }
}

fn ci_for(state: &str) -> CiResult {
    match state {
        "failing" | "failure" => ci_failing(),
        "error" => CiResult {
            state: CiState::Error,
            failed_checks: vec!["build".to_string()],
        },
        "successful" | "success" => ci_success(),
        "pending" => CiResult {
            state: CiState::Pending,
            failed_checks: Vec::new(),
        },
        other => panic!("unknown CI state {other}"),
    }
}

fn capture(pattern: &str, text: &str) -> Option<Vec<String>> {
    let regex = Regex::new(pattern).expect("regex");
    regex.captures(text).map(|captures| {
        captures
            .iter()
            .skip(1)
            .map(|capture| capture.expect("capture").as_str().to_string())
            .collect()
    })
}

fn login_for(label: &str) -> String {
    match label {
        "work" => "octo-work".to_string(),
        "personal" => "octo-personal".to_string(),
        "home" => "octo-home".to_string(),
        other => format!("octo-{other}"),
    }
}

#[given(regex = r#"^a configured account "([^"]+)" authenticating through gh as "([^"]+)"$"#)]
async fn gh_account(world: &mut World, label: String, login: String) {
    world.add_gh_account(&label, &login, false);
}

#[given(regex = r#"^gh is authenticated as "([^"]+)"$"#)]
async fn gh_authenticated(world: &mut World, login: String) {
    world.set_gh_authenticated(&login, true);
}

#[given(regex = r#"^gh is not authenticated as "([^"]+)"$"#)]
async fn gh_not_authenticated(world: &mut World, login: String) {
    world.set_gh_authenticated(&login, false);
}

#[given(regex = r#"^gh login always fails$"#)]
async fn gh_login_fails(world: &mut World) {
    world.gh_login_fails = true;
}

#[given(regex = r#"^the terminal is interactive$"#)]
async fn terminal_interactive(world: &mut World) {
    world.interactive = true;
}

#[given(regex = r#"^the terminal is not interactive$"#)]
async fn terminal_not_interactive(world: &mut World) {
    world.interactive = false;
}

#[given(
    regex = r#"^a configured account "([^"]+)" with a personal access token in a file with mode "([^"]+)"$"#
)]
async fn token_file_with_mode(world: &mut World, label: String, mode: String) {
    let mode = u32::from_str_radix(&mode, 8).expect("file mode");
    world.add_file_account(&label, &login_for(&label), None, mode);
    let path = world.account_token_file_path(&label);
    world.set_config_token_file(&label, &path.display().to_string());
}

#[given(regex = r#"^a configured account "([^"]+)" with a relative token file path$"#)]
async fn token_file_relative(world: &mut World, label: String) {
    world.enable_real_credentials();
    world.set_config_token_file(&label, "relative.token");
}

#[given(regex = r#"^a configured account "([^"]+)" with a token file path that is a directory$"#)]
async fn token_file_directory(world: &mut World, label: String) {
    world.enable_real_credentials();
    let path = world.state_path().with_file_name("token-dir");
    std::fs::create_dir_all(&path).expect("token dir");
    world.set_config_token_file(&label, &path.display().to_string());
}

#[given(regex = r#"^a configured account "([^"]+)" with an empty token file$"#)]
async fn token_file_empty(world: &mut World, label: String) {
    world.enable_real_credentials();
    let path = world.state_path().with_file_name("empty.token");
    std::fs::write(&path, "").expect("empty token file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&path, permissions).expect("mode");
    }
    world.set_config_token_file(&label, &path.display().to_string());
}

#[given(
    regex = r#"^a configured account "([^"]+)" with a personal access token file that does not exist$"#
)]
async fn token_file_missing(world: &mut World, label: String) {
    world.add_account(&label, &login_for(&label), false);
    world.enable_real_credentials();
    world.set_config_missing_token_file(&label);
}

#[given(regex = r#"^a configured account "([^"]+)" with a personal access token in "([^"]+)"$"#)]
async fn token_file_tilde(world: &mut World, label: String, path: String) {
    world.add_account(&label, &login_for(&label), false);
    let token = world.accounts[&label].token.clone();
    let file_name = path.trim_start_matches("~/");
    world.prepare_home_token_file(file_name, &token);
    world.enable_real_credentials();
    world.set_config_token_file(&label, &path);
}

#[given(
    regex = r#"^a configured account "([^"]+)" with an expected login "([^"]+)" and a personal access token belonging to "([^"]+)"$"#
)]
async fn account_expected_login(
    world: &mut World,
    label: String,
    expected: String,
    belonging: String,
) {
    world.add_account(&label, &belonging, false);
    world.set_expected_login(&label, &expected);
    world.enable_real_credentials();
}

#[given(regex = r#"^a configured account "([^"]+)" with an environment credential$"#)]
async fn account_environment(world: &mut World, label: String) {
    world.add_account(&label, &login_for(&label), false);
}

#[given(regex = r#"^a configured account "([^"]+)" with a command credential$"#)]
async fn account_command(world: &mut World, label: String) {
    world.add_account(&label, &login_for(&label), false);
    if let Some(account) = world.accounts.get_mut(&label) {
        account.auth = AccountAuth::Command {
            command: "legacy-command".to_string(),
        };
        account.token = "token-legacy-command".to_string();
    }
}

#[given(
    regex = r#"^a configured account "([^"]+)" with (no credential source|an unknown auth mode|two personal access token sources|gh OAuth but no expected login|gh OAuth and a personal access token source|a literal personal access token)$"#
)]
async fn invalid_credential_config(world: &mut World, label: String, invalid: String) {
    world.set_config_invalid(&label, &invalid);
}

#[when(regex = r#"^the watcher starts interactively$"#)]
async fn watcher_starts_interactively(world: &mut World) {
    world.interactive = true;
    world.run_watch_cycles(1).await;
}

#[when(regex = r#"^the watcher runs one observation cycle$"#)]
async fn watcher_runs_one_cycle(world: &mut World) {
    world.run_watch_cycles(1).await;
}

#[when(regex = r#"^the watcher runs two observation cycles$"#)]
async fn watcher_runs_two_cycles(world: &mut World) {
    world.run_watch_cycles(2).await;
}

#[when(regex = r#"^the credential for "([^"]+)" becomes a token belonging to "([^"]+)"$"#)]
async fn rotate(world: &mut World, label: String, new_login: String) {
    let old_login = world.accounts[&label].login.clone();
    world.rotate_credential(&label, &new_login);
    for pull_request in world.pull_requests.values_mut() {
        if pull_request.author == old_login {
            pull_request.author = new_login.clone();
        }
    }
}

#[then(regex = r#"^gh started a browser login for "([^"]+)"$"#)]
async fn gh_started_login(world: &mut World, login: String) {
    let attempts = world.gh_login_attempts.lock().expect("login attempts");
    assert!(
        attempts.iter().any(|attempt| attempt == &login),
        "expected a browser login for {login}, got {attempts:?}"
    );
}

#[then(regex = r#"^gh did not start a browser login$"#)]
async fn gh_did_not_start_login(world: &mut World) {
    let attempts = world.gh_login_attempts.lock().expect("login attempts");
    assert!(attempts.is_empty(), "unexpected logins: {attempts:?}");
}

#[then(regex = r#"^gh started one browser login for "([^"]+)"$"#)]
async fn gh_started_one_login(world: &mut World, login: String) {
    let attempts = world.gh_login_attempts.lock().expect("login attempts");
    let count = attempts.iter().filter(|attempt| *attempt == &login).count();
    assert_eq!(count, 1, "expected one login for {login}, got {attempts:?}");
}

#[then(
    regex = r#"^the credential value "([^"]+)" appears in neither the transcript nor the state file$"#
)]
async fn token_not_leaked(world: &mut World, token: String) {
    assert!(
        !world.last_transcript.contains(&token),
        "token appeared in the transcript"
    );
    let state = std::fs::read_to_string(world.state_path()).unwrap_or_default();
    assert!(!state.contains(&token), "token appeared in the state file");
}

#[then(regex = r#"^the transcript reports no notification for "([^"]+)"$"#)]
async fn no_notification_for(world: &mut World, key: String) {
    assert!(
        !world
            .last_report()
            .notifications
            .iter()
            .any(|notification| notification.key.to_string() == key),
        "unexpected notification for {key}: {:?}",
        world.last_report().notifications
    );
}

#[when(regex = r#"^the next observation fails$"#)]
async fn next_observation_fails(world: &mut World) {
    world.fail_observe_once = true;
}
