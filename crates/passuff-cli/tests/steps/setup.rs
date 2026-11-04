use cucumber::gherkin::Step;
use cucumber::{given, then, when};

use crate::support::World;

fn path(world: &World) -> std::path::PathBuf {
    world.setup_config_path()
}

#[given(regex = r#"^no configuration file exists$"#)]
async fn no_config(world: &mut World) {
    if world.e2e && world.e2e_config_dir.is_none() {
        world.e2e_config_dir = Some(tempfile::TempDir::new().expect("e2e config dir"));
        world.e2e_state_path = Some(
            world
                .e2e_config_dir
                .as_ref()
                .expect("e2e config dir")
                .path()
                .join("state.json"),
        );
    }
    let path = path(world);
    let _ = std::fs::remove_file(&path);
    world.config_text = None;
    world.setup_original = None;
}

#[given(regex = r#"^a configuration file exists$"#)]
async fn config_exists(world: &mut World) {
    let path = path(world);
    let original = "# original\n".to_string();
    std::fs::write(&path, &original).expect("config file");
    world.setup_original = Some(original);
}

#[given(regex = r#"^a world-readable token file$"#)]
async fn world_readable_token(world: &mut World) {
    let path = world
        .state_dir
        .as_ref()
        .expect("state dir")
        .path()
        .join("token");
    std::fs::write(&path, "token-value").expect("token file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("token permissions");
    }
    world.setup_token_file = Some(path);
}

#[given(regex = r#"^the setup configuration directory cannot be written$"#)]
async fn readonly_config(world: &mut World) {
    let path = path(world);
    let directory = path.parent().expect("config parent").to_path_buf();
    std::fs::create_dir_all(&directory).expect("config dir");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500))
            .expect("readonly config");
    }
}

fn answers(step: &Step, world: &World) -> Vec<String> {
    let table = step.table.as_ref().expect("setup table");
    let mut values: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for row in &table.rows {
        if row.len() == 2 {
            values.insert(row[0].clone(), row[1].clone());
        }
    }
    let mut answers = Vec::new();
    if let Some(replace) = values.get("replace") {
        answers.push(replace.clone());
        if replace != "yes" {
            return answers;
        }
    }
    answers.push(values.get("label").cloned().unwrap_or_default());
    answers.push(
        values
            .get("auth")
            .cloned()
            .unwrap_or_else(|| "gh_oauth".to_string()),
    );
    answers.push(values.get("login").cloned().unwrap_or_default());
    if let Some(variable) = values.get("variable") {
        answers.push(variable.clone());
    }
    if let Some(file) = values.get("file") {
        if file == "TOKEN_FILE" {
            answers.push(
                world
                    .setup_token_file
                    .as_ref()
                    .expect("token file fixture")
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            answers.push(file.clone());
        }
    }
    answers.push("no".to_string());
    answers.push(
        values
            .get("watch")
            .cloned()
            .unwrap_or_else(|| "all".to_string()),
    );
    if let Some(repos) = values.get("repos") {
        answers.push(repos.clone());
    }
    answers
}

#[when(regex = r#"^the developer runs setup with:$"#)]
async fn runs_setup(world: &mut World, step: &Step) {
    let answers = answers(step, world);
    world.run_setup(answers).await;
}

#[when(regex = r#"^the developer runs setup and answers "([^"]+)" to replacement$"#)]
async fn runs_setup_no_replace(world: &mut World, answer: String) {
    world.run_setup(vec![answer]).await;
}

#[then(regex = r#"^the configuration is created$"#)]
async fn created(world: &mut World) {
    assert!(path(world).exists(), "configuration not created");
}

#[then(regex = r#"^the configuration is not created$"#)]
async fn not_created(world: &mut World) {
    assert!(!path(world).exists(), "configuration was created");
}

#[then(regex = r#"^the configuration contains account "([^"]+)"$"#)]
async fn contains_account(world: &mut World, label: String) {
    let config = passuff_config::ConfigLoader::parse(&world.setup_config_text())
        .expect("setup configuration");
    assert!(
        config.core.accounts.contains_key(&label),
        "missing account {label}: {}",
        world.setup_config_text()
    );
}

#[then(regex = r#"^the configuration contains "([^"]+)"$"#)]
async fn contains(world: &mut World, expected: String) {
    let text = world.setup_config_text();
    assert!(text.contains(&expected), "missing {expected:?} in {text}");
}

#[then(regex = r#"^the configuration contains auth "([^"]+)"$"#)]
async fn contains_auth(world: &mut World, expected: String) {
    let text = world.setup_config_text();
    let needle = format!("auth = \"{expected}\"");
    assert!(text.contains(&needle), "missing {needle:?} in {text}");
}

#[then(regex = r#"^the configuration contains token env "([^"]+)"$"#)]
async fn contains_token_env(world: &mut World, expected: String) {
    let text = world.setup_config_text();
    let needle = format!("token_env = \"{expected}\"");
    assert!(text.contains(&needle), "missing {needle:?} in {text}");
}

#[then(regex = r#"^the configuration contains login "([^"]+)"$"#)]
async fn contains_login(world: &mut World, expected: String) {
    let text = world.setup_config_text();
    let needle = format!("github_user = \"{expected}\"");
    assert!(text.contains(&needle), "missing {needle:?} in {text}");
}

#[then(regex = r#"^the configuration does not contain "([^"]+)"$"#)]
async fn does_not_contain(world: &mut World, unexpected: String) {
    let text = world.setup_config_text();
    assert!(
        !text.contains(&unexpected),
        "unexpected {unexpected:?} in {text}"
    );
}

#[then(regex = r#"^the configuration is unchanged$"#)]
async fn unchanged(world: &mut World) {
    assert_eq!(
        world.setup_config_text(),
        world.setup_original.clone().expect("original config")
    );
}

#[then(regex = r#"^setup reports "([^"]+)"$"#)]
async fn reports(world: &mut World, expected: String) {
    assert!(
        world.last_transcript.contains(&expected),
        "expected setup to report {expected:?}: {:?}",
        world.last_transcript
    );
}

#[when(regex = r#"^the developer opens the inbox TUI without a terminal$"#)]
async fn tui_without_terminal(world: &mut World) {
    let config = path(world);
    let state = if world.e2e {
        world.e2e_state_path.clone().expect("e2e state")
    } else {
        world.state_path()
    };
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_passuff-tui"))
        .arg("--snapshot")
        .arg("--config")
        .arg(config)
        .arg("--state")
        .arg(state)
        .output()
        .expect("run passuff-tui");
    world.last_status = output.status.code().unwrap_or(3);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    world.last_transcript = format!("{stdout}{stderr}");
}
