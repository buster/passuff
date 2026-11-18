use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::{Parser, Subcommand};
use passuff_config::AppConfig;
use passuff_core::model::RefreshReport;
use passuff_core::state::{self, LoadedState};
use passuff_core::{PassuffCore, WatcherState};

use crate::delivery::{self, Surfaces};
use crate::paths::Paths;
use crate::render;
use crate::setup;
use crate::sound;
use crate::util;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[derive(Parser, Debug)]
#[command(
    name = "passuff",
    version,
    about = "Multi-account GitHub pull request inbox and desktop notifier",
    long_about = "Watches the pull requests you authored, are assigned to, or must review.\n\
                  Without a subcommand passuff watches until interrupted."
)]
pub struct Cli {
    /// Use a specific configuration file
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<std::path::PathBuf>,
    /// Use a specific state file
    #[arg(long, global = true, value_name = "PATH")]
    pub state: Option<std::path::PathBuf>,
    /// Diagnostic logs on stderr
    #[arg(long, global = true)]
    pub verbose: bool,
    /// Suppress desktop notifications for this run
    #[arg(long = "no-desktop", global = true)]
    pub no_desktop: bool,
    /// Suppress the notification sound for this run
    #[arg(long = "no-sound", global = true)]
    pub no_sound: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run one observation pass and print notifications
    Check,
    /// Show accounts, watch scope, watched pull requests, and unread count
    Status,
    /// List unread activity
    Inbox {
        /// Include read activity
        #[arg(long)]
        all: bool,
    },
    /// Mark one activity record read, or every record with --all
    Read {
        /// Activity id, for example a1
        id: Option<String>,
        /// Mark every activity record read
        #[arg(long)]
        all: bool,
    },
    /// Create the configuration interactively
    Setup,
    /// Send one test notification and play the configured sound
    NotifyTest,
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let paths = Paths::resolve(cli.config.clone(), cli.state.clone());
    util::verbose(
        cli.verbose,
        format!(
            "config {} state {}",
            paths.config.display(),
            paths.state.display()
        ),
    );
    match cli.command {
        None => run_watch(&paths, &cli),
        Some(Command::Check) => run_check(&paths, &cli),
        Some(Command::Status) => run_status(&paths),
        Some(Command::Inbox { all }) => run_inbox(&paths, all),
        Some(Command::Read { id, all }) => run_read(&paths, id.as_deref(), all),
        Some(Command::Setup) => run_setup_command(&paths),
        Some(Command::NotifyTest) => run_notify_test(&paths, &cli),
    }
}

fn load_config(path: &Path) -> Result<AppConfig, i32> {
    match passuff_config::load(path) {
        Ok(config) => Ok(config),
        Err(error) => {
            eprintln!("passuff: configuration error: {error}");
            Err(2)
        }
    }
}

fn load_state(path: &Path) -> Result<LoadedState, i32> {
    let loaded = match state::load(path) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("passuff: state error: {error}");
            return Err(3);
        }
    };
    if loaded.needs_save {
        if let Err(error) = state::save(path, &loaded.state) {
            eprintln!("passuff: state error: {error}");
            return Err(3);
        }
    }
    Ok(loaded)
}

fn save_state(path: &Path, state: &WatcherState) -> Result<(), i32> {
    state::save(path, state).map_err(|error| {
        eprintln!("passuff: state error: {error}");
        3
    })
}

fn surfaces(config: &AppConfig, no_desktop: bool, no_sound: bool) -> Surfaces {
    Surfaces {
        desktop: config.notifications.desktop && !no_desktop,
        sound: config.notifications.sound && !no_sound,
        urgency: config.notifications.urgency,
        sound_file: config.notifications.sound_file.clone(),
    }
}

pub fn run_check(paths: &Paths, cli: &Cli) -> i32 {
    let config = match load_config(&paths.config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let mut loaded = match load_state(&paths.state) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let mut core = PassuffCore::new();
    let report = core.refresh_targeted(&config, &mut loaded.state, false);
    if let Err(code) = save_state(&paths.state, &loaded.state) {
        return code;
    }
    print_report(&report);
    deliver_report(&config, cli, &report);
    if report.has_failures() {
        1
    } else {
        0
    }
}

fn print_report(report: &RefreshReport) {
    println!("{}", render::check_transcript(report));
    for failure in &report.failures {
        eprintln!("{}", render::account_failure_line(failure));
    }
}

fn deliver_report(config: &AppConfig, cli: &Cli, report: &RefreshReport) {
    let Some(digest) = render::digest(&report.notifications) else {
        return;
    };
    let result = delivery::deliver(&digest, &surfaces(config, cli.no_desktop, cli.no_sound));
    if let Some(error) = result.desktop_error {
        eprintln!("passuff: desktop notification failed: {error}");
    }
    if let Some(error) = result.sound_error {
        eprintln!("passuff: {error}");
    }
}

pub fn run_status(paths: &Paths) -> i32 {
    let config = match load_config(&paths.config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let loaded = match load_state(&paths.state) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    println!("{}", render::status(&config, &loaded.state));
    0
}

pub fn run_inbox(paths: &Paths, all: bool) -> i32 {
    let config = match load_config(&paths.config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let loaded = match load_state(&paths.state) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    println!("{}", render::inbox(&config, &loaded.state, all));
    0
}

pub fn run_read(paths: &Paths, id: Option<&str>, all: bool) -> i32 {
    if all == id.is_some() {
        eprintln!("passuff: read requires exactly one of <id> or --all");
        return 2;
    }
    if let Err(code) = load_config(&paths.config) {
        return code;
    }
    let mut loaded = match load_state(&paths.state) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    let changed = if all {
        let mut changed = 0;
        for record in &mut loaded.state.activity {
            if !record.read {
                record.read = true;
                changed += 1;
            }
        }
        changed
    } else {
        let id = id.unwrap_or_default();
        match loaded.state.find_activity_mut(id) {
            Some(record) => {
                if record.read {
                    0
                } else {
                    record.read = true;
                    1
                }
            }
            None => {
                eprintln!("passuff: no activity with id {id}");
                return 2;
            }
        }
    };

    if changed > 0 || loaded.existed {
        if let Err(code) = save_state(&paths.state, &loaded.state) {
            return code;
        }
    }
    println!("{}", render::read_message(changed));
    0
}

fn run_setup_command(paths: &Paths) -> i32 {
    match setup::run(&paths.config) {
        Ok(_) => 0,
        Err(setup::SetupError::Validation(message)) => {
            println!("passuff: {message}");
            2
        }
        Err(setup::SetupError::Cancelled) => {
            println!("passuff: setup cancelled");
            2
        }
        Err(setup::SetupError::Io(message)) => {
            println!("passuff: cannot write configuration: {message}");
            3
        }
    }
}

fn run_notify_test(paths: &Paths, cli: &Cli) -> i32 {
    let config = match load_config(&paths.config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let desktop = config.notifications.desktop && !cli.no_desktop;
    let sound_enabled = config.notifications.sound && !cli.no_sound;
    let mut failed = false;

    if desktop {
        match delivery::send_desktop(
            "passuff",
            "If you can read this, passuff notifications are working.",
            config.notifications.urgency,
        ) {
            Ok(()) => println!("passuff: desktop notification sent"),
            Err(error) => {
                println!("passuff: desktop notification failed: {error}");
                failed = true;
            }
        }
    } else {
        println!("passuff: desktop notifications are disabled");
    }

    if sound_enabled {
        match sound::play(&config.notifications.sound_file) {
            Ok(()) => println!("passuff: sound played"),
            Err(error) => {
                println!("passuff: {error}");
                failed = true;
            }
        }
    } else {
        println!("passuff: sound is disabled");
    }

    if !desktop && !sound_enabled {
        println!("passuff: nothing to test");
        return 2;
    }
    if failed {
        1
    } else {
        0
    }
}

pub fn run_watch(paths: &Paths, cli: &Cli) -> i32 {
    let mut config = match load_config(&paths.config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let mut loaded = match load_state(&paths.state) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    let _ = ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    });
    INTERRUPTED.store(false, Ordering::SeqCst);

    println!("passuff: watching");
    let interactive = util::terminal_interactive();
    let mut core = PassuffCore::new();

    loop {
        if INTERRUPTED.load(Ordering::SeqCst) {
            return 0;
        }

        match passuff_config::load(&paths.config) {
            Ok(reloaded) => config = reloaded,
            Err(error) => {
                eprintln!("passuff: configuration error: {error}");
            }
        }

        let report = core.refresh_targeted(&config, &mut loaded.state, interactive);
        if let Err(code) = save_state(&paths.state, &loaded.state) {
            return code;
        }
        print_report(&report);
        deliver_report(&config, cli, &report);
        let _ = std::io::Write::flush(&mut std::io::stdout());

        if INTERRUPTED.load(Ordering::SeqCst) {
            return 0;
        }
        let wait = std::time::Duration::from_secs(config.poll_interval_secs.max(1));
        let deadline = std::time::Instant::now() + wait;
        while std::time::Instant::now() < deadline {
            if INTERRUPTED.load(Ordering::SeqCst) {
                return 0;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
