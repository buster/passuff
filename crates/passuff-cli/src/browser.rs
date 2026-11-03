//! Launch the platform browser for a pull request URL.

use std::process::Command;

/// Open a URL in the platform browser.
pub trait BrowserOpener: Send + Sync {
    fn open(&self, url: &str) -> Result<(), String>;
}

/// The system opener: `xdg-open` on Linux, `open` on macOS.
#[derive(Debug, Clone)]
pub struct SystemBrowserOpener {
    program: String,
}

impl SystemBrowserOpener {
    pub fn new() -> Self {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        Self {
            program: program.to_string(),
        }
    }

    /// Use a specific opener program; the test seam for missing and failing
    /// openers.
    pub fn with_program(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for SystemBrowserOpener {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserOpener for SystemBrowserOpener {
    fn open(&self, url: &str) -> Result<(), String> {
        let status = Command::new(&self.program)
            .arg(url)
            .status()
            .map_err(|error| format!("{}: {error}", self.program))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "{} exited {}",
                self.program,
                status.code().unwrap_or(-1)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_opener_is_reported() {
        let opener = SystemBrowserOpener::with_program("passuff-missing-opener");
        let error = opener
            .open("https://github.com/acme/api/pull/42")
            .expect_err("missing opener");
        assert!(error.contains("passuff-missing-opener"), "{error}");
    }

    #[test]
    fn a_non_zero_exit_is_reported() {
        let opener = SystemBrowserOpener::with_program("/bin/false");
        let error = opener
            .open("https://github.com/acme/api/pull/42")
            .expect_err("failing opener");
        assert!(error.contains("exited"), "{error}");
    }
}
