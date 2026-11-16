use std::io::{BufRead, Write};
use std::path::Path;

use passuff_config::{
    build_setup_config, create_config, validate_repository, AuthMode, ConfigError, SetupAccount,
    SetupAnswers, WatchConfig, WatchMode,
};

pub struct SetupResult {
    pub exit_code: i32,
    pub stdout: String,
}

struct TeeWriter<'a> {
    primary: &'a mut dyn Write,
    buffer: Vec<u8>,
}

impl Write for TeeWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let _ = self.primary.write_all(bytes);
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.primary.flush()
    }
}

pub fn run_setup(
    config_path: &Path,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> SetupResult {
    let mut tee = TeeWriter {
        primary: output,
        buffer: Vec::new(),
    };
    let result = setup_inner(config_path, input, &mut tee);
    let exit_code = match result {
        SetupOutcome::Created | SetupOutcome::Unchanged => 0,
        SetupOutcome::Failed { message, exit_code } => {
            let _ = writeln!(tee, "passuff setup: {message}");
            exit_code
        }
    };
    let _ = tee.flush();
    SetupResult {
        exit_code,
        stdout: String::from_utf8_lossy(&tee.buffer).to_string(),
    }
}

enum SetupOutcome {
    Created,
    Unchanged,
    Failed { message: String, exit_code: i32 },
}

fn setup_inner<W: Write>(
    config_path: &Path,
    input: &mut dyn BufRead,
    output: &mut W,
) -> SetupOutcome {
    if config_path.exists() {
        let answer = match prompt(input, output, "Configuration exists. Replace? [y/N]: ") {
            Ok(answer) => answer,
            Err(_) => {
                return SetupOutcome::Failed {
                    message: "setup cancelled".to_string(),
                    exit_code: 2,
                }
            }
        };
        if !matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") {
            let _ = writeln!(output, "passuff setup: configuration left unchanged");
            return SetupOutcome::Unchanged;
        }
    }

    let mut accounts = Vec::new();
    loop {
        let label = loop {
            match prompt(input, output, "Account label: ") {
                Ok(value) if !value.trim().is_empty() => break value.trim().to_string(),
                Ok(_) => {
                    let _ = writeln!(output, "passuff setup: account label must not be empty");
                }
                Err(message) => {
                    return SetupOutcome::Failed {
                        message,
                        exit_code: 2,
                    }
                }
            }
        };
        let auth = loop {
            match prompt(
                input,
                output,
                "Auth [gh_oauth/token_env/token_command/token_file] (gh_oauth): ",
            ) {
                Ok(value) => {
                    let value = value.trim().to_ascii_lowercase();
                    let mode = match value.as_str() {
                        "" | "gh_oauth" | "gh-oauth" => Some(AuthMode::GhOAuth),
                        "token_env" | "env" => Some(AuthMode::GithubPat),
                        "token_command" | "command" => Some(AuthMode::GithubPat),
                        "token_file" | "file" => Some(AuthMode::GithubPat),
                        _ => None,
                    };
                    if let Some(mode) = mode {
                        let value = if value.is_empty() {
                            "gh_oauth".to_string()
                        } else {
                            value
                        };
                        break (mode, value);
                    }
                    let _ = writeln!(output, "passuff setup: unknown auth mode");
                }
                Err(message) => {
                    return SetupOutcome::Failed {
                        message,
                        exit_code: 2,
                    }
                }
            }
        };
        let login = loop {
            match prompt(input, output, "Expected GitHub login: ") {
                Ok(value)
                    if !value.trim().is_empty() && !value.chars().any(char::is_whitespace) =>
                {
                    break value.trim().to_string()
                }
                Ok(_) => {
                    let _ = writeln!(output, "passuff setup: github_user requires a value");
                }
                Err(message) => {
                    return SetupOutcome::Failed {
                        message,
                        exit_code: 2,
                    }
                }
            }
        };
        let token_source = if auth.0 == AuthMode::GhOAuth {
            None
        } else {
            let prompt_text = match auth.1.as_str() {
                "token_env" | "env" => "Environment variable: ",
                "token_command" | "command" => "Command: ",
                _ => "Token file: ",
            };
            loop {
                match prompt(input, output, prompt_text) {
                    Ok(value) if !value.trim().is_empty() => {
                        let prefix = match auth.1.as_str() {
                            "token_env" | "env" => "env",
                            "token_command" | "command" => "command",
                            _ => "file",
                        };
                        let candidate = format!("{prefix}:{}", value.trim());
                        if prefix == "file" {
                            let provisional = SetupAnswers {
                                accounts: vec![SetupAccount {
                                    label: label.clone(),
                                    auth: AuthMode::GithubPat,
                                    login: login.clone(),
                                    token_source: Some(candidate.clone()),
                                }],
                                watch: WatchConfig::default(),
                            };
                            if let Err(error) = build_setup_config(&provisional) {
                                let _ = writeln!(output, "passuff setup: {error}");
                                continue;
                            }
                        }
                        break Some(candidate);
                    }
                    Ok(_) => {
                        let _ =
                            writeln!(output, "passuff setup: credential source requires a value");
                    }
                    Err(message) => {
                        return SetupOutcome::Failed {
                            message,
                            exit_code: 2,
                        }
                    }
                }
            }
        };
        accounts.push(SetupAccount {
            label,
            auth: auth.0,
            login,
            token_source,
        });
        match prompt(input, output, "Add another account? [y/N]: ") {
            Ok(answer) if matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") => {}
            Ok(_) => break,
            Err(message) => {
                return SetupOutcome::Failed {
                    message,
                    exit_code: 2,
                }
            }
        }
    }

    let mode = loop {
        match prompt(input, output, "Watch mode [all/selected] (all): ") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "" | "all" => break WatchMode::All,
                "selected" => break WatchMode::Selected,
                _ => {
                    let _ = writeln!(output, "passuff setup: watch mode must be all or selected");
                }
            },
            Err(message) => {
                return SetupOutcome::Failed {
                    message,
                    exit_code: 2,
                }
            }
        }
    };
    let mut repositories = Vec::new();
    if mode == WatchMode::Selected {
        loop {
            let value = match prompt(input, output, "Repositories (comma-separated): ") {
                Ok(value) => value,
                Err(message) => {
                    return SetupOutcome::Failed {
                        message,
                        exit_code: 2,
                    }
                }
            };
            let mut parsed = Vec::new();
            let mut valid = true;
            for repository in value
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
            {
                if let Err(error) = validate_repository(repository) {
                    let _ = writeln!(output, "passuff setup: {error}");
                    valid = false;
                    break;
                }
                if parsed
                    .iter()
                    .any(|candidate: &String| candidate.eq_ignore_ascii_case(repository))
                {
                    let _ = writeln!(
                        output,
                        "passuff setup: duplicate watch repository: {repository}"
                    );
                    valid = false;
                    break;
                }
                parsed.push(repository.to_string());
            }
            if valid {
                repositories = parsed;
                break;
            }
        }
    }

    let answers = SetupAnswers {
        accounts,
        watch: WatchConfig {
            mode: Some(mode),
            repositories,
            ignored_pull_requests: Vec::new(),
        },
    };
    if let Err(error) = build_setup_config(&answers) {
        return SetupOutcome::Failed {
            message: error.to_string(),
            exit_code: 2,
        };
    }
    match create_config(config_path, &answers) {
        Ok(_) => {
            let _ = writeln!(
                output,
                "passuff setup: configuration written to {}",
                config_path.display()
            );
            SetupOutcome::Created
        }
        Err(ConfigError::Write { .. }) => SetupOutcome::Failed {
            message: format!("cannot write configuration {}", config_path.display()),
            exit_code: 3,
        },
        Err(error) => SetupOutcome::Failed {
            message: error.to_string(),
            exit_code: 2,
        },
    }
}

fn prompt<W: Write>(input: &mut dyn BufRead, output: &mut W, text: &str) -> Result<String, String> {
    let _ = output.write_all(text.as_bytes());
    let _ = output.flush();
    let mut line = String::new();
    match input.read_line(&mut line) {
        Ok(0) => Err("setup cancelled".to_string()),
        Ok(_) => Ok(line.trim_end_matches(['\r', '\n']).to_string()),
        Err(error) => Err(format!("cannot read setup input: {error}")),
    }
}
