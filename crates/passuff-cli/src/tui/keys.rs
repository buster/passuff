#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Tab,
    BackTab,
    Down,
    Up,
    PageDown,
    PageUp,
    Home,
    End,
    Enter,
    Esc,
    Quit,
    Left,
    Right,
    Space,
    Help,
    MarkRead,
    MarkAllRead,
    ToggleUnread,
    ClearFilter,
    ResetView,
    SortNext,
    SortReverse,
    FilterPullRequest,
    FilterStack,
    ToggleIgnore,
    ToggleIgnoredVisibility,
    OpenSettings,
    OpenPicker,
    ToggleMode,
    Char(char),
}

pub fn parse_keys(value: &str) -> Result<Vec<Key>, String> {
    let mut keys = Vec::new();
    for raw in value.split(',') {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let key = match raw {
            "S" => Key::SortReverse,
            "F" => Key::FilterStack,
            "C" => Key::ResetView,
            "H" => Key::Help,
            _ => match raw.to_ascii_lowercase().as_str() {
                "tab" => Key::Tab,
                "backtab" => Key::BackTab,
                "down" | "j" => Key::Down,
                "up" | "k" => Key::Up,
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
                "home" => Key::Home,
                "end" => Key::End,
                "enter" => Key::Enter,
                "esc" => Key::Esc,
                "q" => Key::Quit,
                "left" => Key::Left,
                "right" => Key::Right,
                "space" => Key::Space,
                "comma" => Key::Char(','),
                "h" | "f1" => Key::Help,
                "r" => Key::MarkRead,
                "a" => Key::MarkAllRead,
                "u" => Key::ToggleUnread,
                "c" => Key::ClearFilter,
                "x" => Key::ToggleIgnore,
                "s" => Key::SortNext,
                "f" => Key::FilterPullRequest,
                "p" => Key::OpenPicker,
                "m" => Key::ToggleMode,
                "y" => Key::Char('y'),
                "n" => Key::Char('n'),
                _ => {
                    if raw.chars().count() == 1 {
                        Key::Char(raw.chars().next().unwrap_or_default())
                    } else {
                        return Err(format!("unknown key: {raw}"));
                    }
                }
            },
        };
        keys.push(key);
    }
    Ok(keys)
}
