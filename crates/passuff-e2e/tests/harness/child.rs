//! A long-running product process: the default watch command, its signal, and
//! its exit code. Output is discarded because the scenario asserts the state
//! file and the notification log instead.

use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    pub fn spawn(command: &mut Command) -> Result<Self, String> {
        let child = command
            .spawn()
            .map_err(|error| format!("cannot start the product: {error}"))?;
        Ok(Self { child })
    }

    /// Send a signal through `/bin/kill`; the product installs no handler the
    /// harness could call in process.
    pub fn signal(&self, name: &str) -> Result<(), String> {
        let status = Command::new("/bin/kill")
            .arg(format!("-{name}"))
            .arg(self.child.id().to_string())
            .status()
            .map_err(|error| format!("cannot send SIG{name}: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("/bin/kill exited with {:?}", status.code()))
        }
    }

    pub fn wait_exit(&mut self, timeout: Duration) -> Option<i32> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return Some(status.code().unwrap_or(-1)),
                Ok(None) => {}
                Err(_) => return None,
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
