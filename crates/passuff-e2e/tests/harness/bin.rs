//! Locate the built product binaries and build a clean child environment.
//!
//! The package holds no production dependency, so there is no in-process path
//! to fall back to: an unset or incomplete `PASSUFF_BIN_DIR` is a hard error.

use std::path::PathBuf;
use std::process::Command;

use portable_pty::CommandBuilder;

pub const TOKEN_ENV: &str = "PASSUFF_E2E_TOKEN";
pub const TOKEN: &str = "e2e-token";
pub const PERSONAL_TOKEN_ENV: &str = "PASSUFF_E2E_PERSONAL_TOKEN";
pub const PERSONAL_TOKEN: &str = "e2e-personal-token";
const PATH: &str = "/usr/local/bin:/usr/bin:/bin";
const BINARIES: [&str; 2] = ["passuff", "passuff-tui"];

pub fn bin_dir_from_env() -> Result<PathBuf, String> {
    match std::env::var_os("PASSUFF_BIN_DIR") {
        Some(value) if !value.is_empty() => Ok(PathBuf::from(value)),
        _ => Err(
            "PASSUFF_BIN_DIR is not set; run `cargo build --locked -p passuff --bins` and \
             point PASSUFF_BIN_DIR at the output directory"
                .to_string(),
        ),
    }
}

pub fn verify_binaries() -> Result<(), String> {
    let dir = bin_dir_from_env()?;
    for name in BINARIES {
        let path = dir.join(name);
        if !path.is_file() {
            return Err(format!(
                "PASSUFF_BIN_DIR does not contain {name}: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn program(name: &str) -> Result<PathBuf, String> {
    if !BINARIES.contains(&name) {
        return Err(format!("unknown product binary: {name}"));
    }
    let dir = bin_dir_from_env()?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(format!(
            "PASSUFF_BIN_DIR does not contain {name}: {}",
            path.display()
        ));
    }
    Ok(path)
}

/// The environment every child process starts from. `env_clear` removes the
/// test process environment, then only the declared keys and the per-scenario
/// `PASSUFF_*` hooks are added.
#[derive(Debug, Clone)]
pub struct ChildEnv {
    pub home: PathBuf,
    pub hooks: Vec<(String, String)>,
}

impl ChildEnv {
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            hooks: Vec::new(),
        }
    }

    pub fn with_hook(mut self, key: &str, value: impl Into<String>) -> Self {
        self.hooks.push((key.to_string(), value.into()));
        self
    }
}

pub fn product_command(name: &str, env: &ChildEnv) -> Result<Command, String> {
    let mut command = Command::new(program(name)?);
    command.env_clear();
    command.env("HOME", &env.home);
    command.env("PATH", PATH);
    command.env(TOKEN_ENV, TOKEN);
    command.env(PERSONAL_TOKEN_ENV, PERSONAL_TOKEN);
    for (key, value) in &env.hooks {
        command.env(key, value);
    }
    Ok(command)
}

pub fn pty_command(name: &str, env: &ChildEnv) -> Result<CommandBuilder, String> {
    let mut command = CommandBuilder::new(program(name)?);
    command.env_clear();
    command.env("HOME", &env.home);
    command.env("PATH", PATH);
    command.env(TOKEN_ENV, TOKEN);
    command.env(PERSONAL_TOKEN_ENV, PERSONAL_TOKEN);
    for (key, value) in &env.hooks {
        command.env(key, value);
    }
    Ok(command)
}
