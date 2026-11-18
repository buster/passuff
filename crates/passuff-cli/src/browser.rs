//! Opening a recorded pull request in the platform browser.

use std::process::Command;

pub fn open(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let program = "xdg-open";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = url;
        return Err("could not open browser: unsupported platform".to_string());
    }

    let status = Command::new(program)
        .arg(url)
        .status()
        .map_err(|error| format!("could not open browser: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "could not open browser: {program} exited with {status}"
        ))
    }
}
