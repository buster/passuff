//! Real terminal session: a pty at a fixed size, a reader thread feeding
//! `vt100::Parser`, and assertions against the parsed screen.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};

pub struct PtySession {
    child: Box<dyn Child + Send + Sync>,
    writer: Option<Box<dyn Write + Send>>,
    parser: Arc<Mutex<vt100::Parser>>,
    reader: Option<std::thread::JoinHandle<()>>,
    exit_code: Option<u32>,
}

impl PtySession {
    pub fn spawn(program: CommandBuilder, cols: u16, rows: u16) -> Result<Self, String> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("cannot open pty {cols}x{rows}: {error}"))?;
        let child = pair
            .slave
            .spawn_command(program)
            .map_err(|error| format!("cannot start the product on the pty: {error}"))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("cannot read the pty: {error}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| format!("cannot write the pty: {error}"))?;
        drop(pair.slave);

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let thread_parser = parser.clone();
        let handle = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => {
                        if let Ok(mut parser) = thread_parser.lock() {
                            parser.process(&buffer[..count]);
                        }
                    }
                }
            }
        });
        Ok(Self {
            child,
            writer: Some(writer),
            parser,
            reader: Some(handle),
            exit_code: None,
        })
    }

    pub fn screen(&self) -> String {
        match self.parser.lock() {
            Ok(parser) => parser.screen().contents(),
            Err(_) => String::new(),
        }
    }

    /// Wait until the screen contains `needle`. Returns the final screen.
    pub fn wait_for(&self, needle: &str, timeout: Duration) -> String {
        let deadline = Instant::now() + timeout;
        loop {
            let screen = self.screen();
            if screen.contains(needle) {
                return screen;
            }
            if Instant::now() >= deadline {
                return screen;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| "the pty writer is closed".to_string())?;
        writer
            .write_all(bytes)
            .and_then(|()| writer.flush())
            .map_err(|error| format!("cannot write to the pty: {error}"))
    }

    pub fn wait_exit(&mut self, timeout: Duration) -> Option<u32> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(code) = self.exit_code {
                return Some(code);
            }
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.exit_code();
                    self.exit_code = Some(code);
                    return Some(code);
                }
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

impl Drop for PtySession {
    fn drop(&mut self) {
        // Dropping the writer sends EOF to the child stdin; killing the child
        // then closes the slave side and unblocks the reader thread.
        self.writer.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(handle) = self.reader.take() {
            let _ = handle.join();
        }
    }
}
