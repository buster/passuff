//! Interactive first-run setup. The flow never reads or writes a token value.

use std::io::{BufRead, Write};
use std::path::Path;

use passuff_config::{AccountConfig, AppConfig, AuthMode, WatchConfig, WatchMode};

pub enum SetupOutcome {
    Created,
    Unchanged,
}

pub enum SetupError {
    Validation(String),
    Cancelled,
    Io(String),
}

pub fn run(path: &Path) -> Result<SetupOutcome, SetupError> {
    if path.exists() {
        print!("Configuration exists. Replace? [y/N] ");
        let _ = std::io::stdout().flush();
        let answer = read_line()?.unwrap_or_default();
        if !matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") {
            return Ok(SetupOutcome::Unchanged);
        }
    }

    let mut config = AppConfig::default();
    let mut account_number = 0usize;
    loop {
        account_number += 1;
        let (label, account) = prompt_account(account_number)?;
        config.accounts.insert(label, account);
        print!("Add another account? [y/N] ");
        let _ = std::io::stdout().flush();
        let answer = read_line()?.unwrap_or_default();
        if !matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") {
            break;
        }
    }

    let mode = prompt_mode()?;
    let repositories = if matches!(mode, WatchMode::Selected) {
        prompt_repositories()?
    } else {
        Vec::new()
    };
    config.watch = WatchConfig {
        mode: Some(mode),
        repositories,
        ignored_pull_requests: Vec::new(),
    };

    passuff_config::validate(&config).map_err(SetupError::Validation)?;
    passuff_config::save(path, &config).map_err(|error| match error {
        passuff_config::SaveError::Invalid(message) => SetupError::Validation(message),
        passuff_config::SaveError::Io { source, .. } => SetupError::Io(source.to_string()),
    })?;
    Ok(SetupOutcome::Created)
}

fn prompt_account(number: usize) -> Result<(String, AccountConfig), SetupError> {
    let label = loop {
        let label = prompt(&format!("Account {number} label: "))?;
        if label.is_empty() {
            println!("passuff: account label must not be empty");
            continue;
        }
        break label;
    };

    let auth = loop {
        let answer = prompt("Auth [gh_oauth/token_env/token_command/token_file] (gh_oauth): ")?;
        let answer = if answer.is_empty() {
            "gh_oauth".to_string()
        } else {
            answer
        };
        match answer.as_str() {
            "gh_oauth" | "token_env" | "token_command" | "token_file" => break answer,
            other => println!("passuff: unknown auth mode: {other}"),
        }
    };

    let login = loop {
        let login = prompt("Expected GitHub login: ")?;
        if login.is_empty() {
            println!("passuff: github_user must not be empty");
            continue;
        }
        if login.chars().any(char::is_whitespace) {
            println!("passuff: github_user must not contain whitespace");
            continue;
        }
        break login;
    };

    let mut account = AccountConfig {
        auth: Some(if auth == "gh_oauth" {
            AuthMode::GhOauth.as_str().to_string()
        } else {
            AuthMode::GithubPat.as_str().to_string()
        }),
        github_user: Some(login),
        ..AccountConfig::default()
    };

    match auth.as_str() {
        "gh_oauth" => {}
        "token_env" => loop {
            let variable = prompt("Token environment variable: ")?;
            if variable.is_empty() {
                println!("passuff: token_env must not be empty");
                continue;
            }
            account.token_env = Some(variable);
            break;
        },
        "token_command" => loop {
            let command = prompt("Token helper command: ")?;
            if command.is_empty() {
                println!("passuff: token_command must not be empty");
                continue;
            }
            account.token_command = Some(command);
            break;
        },
        _ => loop {
            let file = prompt("Token file: ")?;
            match validate_setup_token_file(&file) {
                Ok(()) => {
                    account.token_file = Some(file);
                    break;
                }
                Err(message) => println!("passuff: {message}"),
            }
        },
    }

    Ok((label, account))
}

fn validate_setup_token_file(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("token file must not be empty".to_string());
    }
    if !(Path::new(value).is_absolute() || value.starts_with('~')) {
        return Err(format!(
            "token_file must be absolute or start with ~: {value}"
        ));
    }
    let path = passuff_config::expand_home(value);
    let metadata = std::fs::metadata(&path)
        .map_err(|_| format!("token file is missing: {}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("token file is not a regular file: {value}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        if mode != 0o600 && mode != 0o400 {
            return Err(format!(
                "token_file is not private (insecure permissions {mode:04o}): {value}"
            ));
        }
    }
    Ok(())
}

fn prompt_mode() -> Result<WatchMode, SetupError> {
    loop {
        let answer = prompt("Watch mode [all/selected] (all): ")?;
        match answer.to_ascii_lowercase().as_str() {
            "" | "all" => return Ok(WatchMode::All),
            "selected" => return Ok(WatchMode::Selected),
            other => println!("passuff: unknown watch mode: {other}"),
        }
    }
}

fn prompt_repositories() -> Result<Vec<String>, SetupError> {
    loop {
        let answer = prompt("Repositories (comma-separated): ")?;
        let mut repositories = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        let mut failed = None;
        if !answer.trim().is_empty() {
            for raw in answer.split(',') {
                let repository = raw.trim();
                if let Err(message) = passuff_config::validate_repository(repository) {
                    failed = Some(message);
                    break;
                }
                if !seen.insert(repository.to_ascii_lowercase()) {
                    failed = Some(format!("duplicate watch repository: {repository}"));
                    break;
                }
                repositories.push(repository.to_string());
            }
        }
        match failed {
            Some(message) => println!("passuff: {message}"),
            None => return Ok(repositories),
        }
    }
}

fn prompt(label: &str) -> Result<String, SetupError> {
    print!("{label}");
    let _ = std::io::stdout().flush();
    match read_line()? {
        Some(line) => Ok(line),
        None => Err(SetupError::Cancelled),
    }
}

fn read_line() -> Result<Option<String>, SetupError> {
    let mut line = String::new();
    let read = std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|error| SetupError::Io(error.to_string()))?;
    if read == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}
