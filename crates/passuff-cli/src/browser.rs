//! Launch the platform browser for a pull request URL.

use std::process::Command;

/// Open a URL in the platform browser.
pub trait BrowserOpener: Send + Sync {
    fn open(&self, url: &str) -> Result<(), String>;
}

/// The system opener: `xdg-open` on Linux, `open` on macOS.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemBrowserOpener;

impl BrowserOpener for SystemBrowserOpener {
    fn open(&self, url: &str) -> Result<(), String> {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let status = Command::new(program)
            .arg(url)
            .status()
            .map_err(|error| format!("{program}: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "{program} exited {}",
                status.code().unwrap_or(-1)
            ))
        }
    }
}
