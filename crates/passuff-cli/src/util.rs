use std::io::IsTerminal;

use chrono::{DateTime, SecondsFormat, Utc};

pub fn rfc3339(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn short_time(time: DateTime<Utc>) -> String {
    time.format("%m-%d %H:%M").to_string()
}

pub fn terminal_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn truncate_chars(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    value.chars().take(width).collect()
}

pub fn pad(value: &str, width: usize) -> String {
    let length = value.chars().count();
    if length >= width {
        return truncate_chars(value, width);
    }
    format!("{value}{}", " ".repeat(width - length))
}

pub fn fit(value: &str, width: usize) -> String {
    truncate_chars(value, width)
}

pub fn verbose_enabled(flag: bool) -> bool {
    flag || std::env::var_os("RUST_LOG").is_some()
}

pub fn verbose(flag: bool, message: impl std::fmt::Display) {
    if verbose_enabled(flag) {
        eprintln!("passuff: debug: {message}");
    }
}

pub fn activity_word(count: usize) -> &'static str {
    if count == 1 {
        "activity"
    } else {
        "activities"
    }
}

pub fn record_word(count: usize) -> &'static str {
    if count == 1 {
        "record"
    } else {
        "records"
    }
}
