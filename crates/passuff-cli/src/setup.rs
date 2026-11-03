use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use passuff_config::ConfigLoader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupError {
    pub message: String,
    pub exit: i32,
}

impl SetupError {
    fn new(message: impl Into<String>, exit: i32) -> Self {
        Self {
            message: message.into(),
            exit,
        }
    }
}

#[derive(Debug)]
struct Account {
    label: String,
    auth: String,
    login: String,
    token: Option<(String, String)>,
}

pub struct SetupSession<'a, R: BufRead, W: Write> {
    path: PathBuf,
    input: R,
    output: &'a mut W,
}

impl<'a, R: BufRead, W: Write> SetupSession<'a, R, W> {
    pub fn new(path: impl Into<PathBuf>, input: R, output: &'a mut W) -> Self {
        Self {
            path: path.into(),
            input,
            output,
        }
    }

    pub fn run(&mut self) -> Result<(), SetupError> {
        let exists = self.path.exists();
        if exists {
            let answer = self.ask("Configuration exists. Replace? [y/N]: ")?;
            if !matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") {
                self.say("passuff: configuration unchanged")?;
                return Ok(());
            }
        }

        let mut accounts = Vec::new();
        loop {
            let label = self.required("Account label: ")?;
            let auth = self.required("Auth (gh_oauth|token_env|token_command|token_file) [gh_oauth]: ")?;
            let auth = if auth.is_empty() {
                "gh_oauth".to_string()
            } else {
                auth
            };
            if !matches!(
                auth.as_str(),
                "gh_oauth" | "token_env" | "token_command" | "token_file"
            ) {
                return Err(self.fail("unknown auth mode", 2));
            }
            let login = self.required("Expected GitHub login: ")?;
            if auth == "gh_oauth" && login.is_empty() {
                return Err(self.fail("gh OAuth requires an expected login", 2));
            }
            let token = match auth.as_str() {
                "token_env" => {
                    let variable = self.required("Token environment variable: ")?;
                    if variable.is_empty() {
                        return Err(self.fail("token variable is required", 2));
                    }
                    Some(("token_env".to_string(), variable))
                }
                "token_command" => {
                    let command = self.required("Token command: ")?;
                    if command.is_empty() {
                        return Err(self.fail("token command is required", 2));
                    }
                    Some(("token_command".to_string(), command))
                }
                "token_file" => {
                    let file = self.required("Token file: ")?;
                    self.check_token_file(&file)?;
                    Some(("token_file".to_string(), file))
                }
                _ => None,
            };
            accounts.push(Account {
                label,
                auth,
                login,
                token,
            });
            let another = self.optional("Add another account? [y/N]: ")?;
            if !matches!(
                another.to_ascii_lowercase().as_str(),
                "y" | "yes"
            ) {
                break;
            }
        }

        let watch = self.required("Watch (all|selected) [all]: ")?;
        let watch = if watch.is_empty() {
            "all".to_string()
        } else {
            watch
        };
        if !matches!(watch.as_str(), "all" | "selected") {
            return Err(self.fail("watch must be all or selected", 2));
        }
        let repositories = if watch == "selected" {
            let line = self.required("Repositories (comma-separated): ")?;
            let mut repositories: Vec<String> = Vec::new();
            for repository in line.split(',').map(str::trim).filter(|r| !r.is_empty()) {
                validate_repository(repository)
                    .map_err(|error| self.fail(error.message, error.exit))?;
                if !repositories
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(repository))
                {
                    repositories.push(repository.to_string());
                }
            }
            if repositories.is_empty() {
                return Err(self.fail("selected watch scope needs at least one repository", 2));
            }
            repositories
        } else {
            Vec::new()
        };

        let text = build_config(&accounts, &watch, &repositories);
        ConfigLoader::parse(&text)
            .map_err(|error| self.fail(format!("configuration error: {error}"), 2))?;
        self.write_config(&text)?;
        self.say(format!(
            "passuff: configuration written to {}",
            self.path.display()
        ))?;
        Ok(())
    }

    fn ask(&mut self, prompt: &str) -> Result<String, SetupError> {
        write!(self.output, "{prompt}").map_err(|error| self.fail(error.to_string(), 3))?;
        self.output
            .flush()
            .map_err(|error| self.fail(error.to_string(), 3))?;
        let mut line = String::new();
        let read = self
            .input
            .read_line(&mut line)
            .map_err(|error| self.fail(error.to_string(), 3))?;
        if read == 0 {
            return Err(self.fail("setup cancelled", 2));
        }
        Ok(line.trim().to_string())
    }

    fn required(&mut self, prompt: &str) -> Result<String, SetupError> {
        self.ask(prompt)
    }

    fn optional(&mut self, prompt: &str) -> Result<String, SetupError> {
        write!(self.output, "{prompt}").map_err(|error| self.fail(error.to_string(), 3))?;
        self.output
            .flush()
            .map_err(|error| self.fail(error.to_string(), 3))?;
        let mut line = String::new();
        let read = self
            .input
            .read_line(&mut line)
            .map_err(|error| self.fail(error.to_string(), 3))?;
        if read == 0 {
            return Ok(String::new());
        }
        Ok(line.trim().to_string())
    }

    fn check_token_file(&mut self, file: &str) -> Result<(), SetupError> {
        let path = expand_home(file);
        let metadata = std::fs::metadata(&path)
            .map_err(|error| self.fail(format!("token file: {error}"), 3))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(self.fail("insecure token file: group or other access", 3));
            }
        }
        Ok(())
    }

    fn write_config(&mut self, text: &str) -> Result<(), SetupError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| self.fail(format!("cannot write configuration: {error}"), 3))?;
        }
        let temporary = self.path.with_extension("toml.tmp");
        std::fs::write(&temporary, text)
            .map_err(|error| self.fail(format!("cannot write configuration: {error}"), 3))?;
        if let Err(error) = std::fs::rename(&temporary, &self.path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(self.fail(format!("cannot write configuration: {error}"), 3));
        }
        Ok(())
    }

    fn say(&mut self, text: impl AsRef<str>) -> Result<(), SetupError> {
        writeln!(self.output, "{}", text.as_ref())
            .map_err(|error| self.fail(error.to_string(), 3))
    }

    fn fail(&mut self, message: impl Into<String>, exit: i32) -> SetupError {
        let message = message.into();
        let _ = writeln!(self.output, "passuff: {message}");
        SetupError::new(message, exit)
    }
}

fn build_config(accounts: &[Account], watch: &str, repositories: &[String]) -> String {
    let mut document = toml_edit::DocumentMut::new();
    let mut watch_table = toml_edit::Table::new();
    watch_table["mode"] = toml_edit::value(watch);
    let mut repos = toml_edit::Array::new();
    for repository in repositories {
        repos.push(repository.clone());
    }
    watch_table["repositories"] = toml_edit::value(repos);
    document["watch"] = toml_edit::Item::Table(watch_table);
    let mut accounts_table = toml_edit::Table::new();
    for account in accounts {
        let mut table = toml_edit::Table::new();
        match account.auth.as_str() {
            "gh_oauth" => {
                table["auth"] = toml_edit::value("gh_oauth");
                table["github_user"] = toml_edit::value(account.login.clone());
            }
            _ => {
                if !account.login.is_empty() {
                    table["github_user"] = toml_edit::value(account.login.clone());
                }
                if let Some((field, value)) = &account.token {
                    table[field] = toml_edit::value(value.clone());
                }
            }
        }
        accounts_table[&account.label] = toml_edit::Item::Table(table);
    }
    document["accounts"] = toml_edit::Item::Table(accounts_table);
    document.to_string()
}

fn validate_repository(repository: &str) -> Result<(), SetupError> {
    let valid = !repository.is_empty()
        && !repository.chars().any(char::is_whitespace)
        && repository.matches('/').count() == 1
        && !repository.starts_with('/')
        && !repository.ends_with('/');
    if !valid {
        return Err(SetupError::new("must be owner/repo", 2));
    }
    Ok(())
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}

pub fn run_scripted(
    path: &Path,
    answers: &[&str],
    output: &mut Vec<u8>,
) -> Result<(), SetupError> {
    let input = std::io::Cursor::new(format!("{}\n", answers.join("\n")));
    SetupSession::new(path, input, output).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(path: &Path, answers: &[&str]) -> (Result<(), SetupError>, String) {
        let input = std::io::Cursor::new(format!("{}\n", answers.join("\n")));
        let mut output = Vec::new();
        let result = SetupSession::new(path, input, &mut output).run();
        (result, String::from_utf8(output).unwrap_or_default())
    }

    #[test]
    fn creates_a_gh_account_without_a_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let (result, _) = run(
            &path,
            &["personal", "gh_oauth", "octo-personal", "no", "all"],
        );
        result.expect("setup");
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("auth = \"gh_oauth\""), "{text}");
        assert!(text.contains("github_user = \"octo-personal\""), "{text}");
        assert!(!text.contains("token"));
    }

    #[test]
    fn creates_a_token_environment_account() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let (result, _) = run(
            &path,
            &["work", "token_env", "octo-work", "WORK_TOKEN", "no", "all"],
        );
        result.expect("setup");
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("token_env = \"WORK_TOKEN\""), "{text}");
        assert!(!text.contains("token-value"));
    }

    #[test]
    fn rejects_an_invalid_repository() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let (result, _) = run(
            &path,
            &["personal", "gh_oauth", "octo-personal", "no", "selected", "acme"],
        );
        let error = result.expect_err("invalid repository");
        assert_eq!(error.message, "must be owner/repo");
        assert!(!path.exists());
    }

    #[test]
    fn preserves_an_existing_configuration_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "original").unwrap();
        let (result, _) = run(&path, &["no"]);
        result.expect("unchanged");
        assert_eq!(std::fs::read_to_string(path).unwrap(), "original");
    }

    #[test]
    fn replaces_an_existing_configuration_after_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "original").unwrap();
        let (result, _) = run(
            &path,
            &["yes", "personal", "gh_oauth", "octo-new", "no", "all"],
        );
        result.expect("replace");
        assert!(std::fs::read_to_string(path)
            .unwrap()
            .contains("github_user = \"octo-new\""));
    }

    #[test]
    fn rejects_an_insecure_token_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let token = dir.path().join("token");
        std::fs::write(&token, "secret").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&token, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        let token = token.to_string_lossy().to_string();
        let (result, _) = run(
            &path,
            &["personal", "token_file", "octo-personal", &token, "no", "all"],
        );
        let error = result.expect_err("insecure");
        assert!(error.message.contains("insecure"), "{error:?}");
        assert!(!path.exists());
    }

    #[test]
    fn a_failed_write_leaves_no_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let read_only = dir.path().join("read-only");
        std::fs::create_dir(&read_only).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&read_only, std::fs::Permissions::from_mode(0o500)).unwrap();
        }
        let path = read_only.join("config.toml");
        let (result, _) = run(
            &path,
            &["personal", "gh_oauth", "octo-personal", "no", "all"],
        );
        let error = result.expect_err("write failure");
        assert!(error.message.contains("cannot write configuration"), "{error:?}");
        assert!(!path.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&read_only, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
}
