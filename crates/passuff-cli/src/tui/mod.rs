//! Full-screen inbox: data model, key handling, live observation, and the
//! frame renderer.

pub mod render;

use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use passuff_config::{AppConfig, Relation, WatchMode};
use passuff_core::model::{
    ActivityRecord, PullRequestKey, RefreshReport, StackInfo, WatchedPullRequest, WatcherState,
};
use passuff_core::{state, view, PassuffCore};

use crate::paths::Paths;
use crate::render as cli_render;
use crate::ui_prefs::{SortMode, UiPrefs};
use crate::{browser, delivery, setup, util};

#[derive(Debug, Parser)]
#[command(
    name = "passuff-tui",
    version,
    about = "Full-screen pull request inbox",
    long_about = "Full-screen inbox over the watched pull requests and retained activity.\n\
                  Observe the configured accounts while open, or print one frame with --snapshot."
)]
pub struct TuiCli {
    #[arg(long, value_name = "PATH")]
    pub config: Option<std::path::PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub state: Option<std::path::PathBuf>,
    /// Print one frame and exit
    #[arg(long)]
    pub snapshot: bool,
    /// Observe once before opening or printing
    #[arg(long)]
    pub observe: bool,
    /// Observe for the given duration before opening or printing
    #[arg(long, value_name = "MILLIS")]
    pub observe_for: Option<u64>,
    /// Add a repository and switch to selected mode
    #[arg(long = "scope-select", value_name = "OWNER/REPO")]
    pub scope_select: Vec<String>,
    /// Remove a repository from the selected scope
    #[arg(long = "scope-deselect", value_name = "OWNER/REPO")]
    pub scope_deselect: Vec<String>,
    /// Set the watch mode before running
    #[arg(long = "scope-mode", value_name = "all|selected")]
    pub scope_mode: Option<String>,
    /// Set the focus relations before running
    #[arg(long = "focus-set", value_name = "RELATIONS")]
    pub focus_set: Option<String>,
    /// Start with the watch-scope picker
    #[arg(long)]
    pub picker: bool,
    /// Apply a comma-separated key sequence before --snapshot
    #[arg(long, value_name = "KEYS")]
    pub keys: Option<String>,
    #[arg(long = "no-desktop")]
    pub no_desktop: bool,
    #[arg(long = "no-sound")]
    pub no_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    PullRequests,
    Activity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityFilter {
    PullRequest(String),
    Stack { repository: String, number: u64 },
}

#[derive(Debug, Clone)]
pub enum Overlay {
    PullRequestDetail { key: String },
    ActivityDetail { id: String },
    Help { pane: Pane },
    Settings,
    Picker,
    SavePrompt,
}

#[derive(Debug, Clone)]
pub enum WatchedRow {
    StackHeader { number: u64, base: String },
    Pull(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Tab,
    BackTab,
    Enter,
    Esc,
    Function(u8),
    Char(char),
    CtrlC,
}

static TUI_INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub struct App {
    pub config: AppConfig,
    pub state: WatcherState,
    pub paths: Paths,
    pub prefs: UiPrefs,
    pub focus: Pane,
    pub watched_selection: usize,
    pub activity_selection: usize,
    pub overlay: Option<Overlay>,
    pub overlay_scroll: usize,
    pub status: String,
    pub dirty: bool,
    pub picker_selection: usize,
    pub settings_selection: usize,
    pub size: (u16, u16),
    pub quit: bool,
    pub selected_key: Option<String>,
    pub selected_activity_id: Option<String>,
    pub no_desktop: bool,
    pub no_sound: bool,
    core: PassuffCore,
}

impl App {
    pub fn new(
        config: AppConfig,
        state: WatcherState,
        paths: Paths,
        prefs: UiPrefs,
        no_desktop: bool,
        no_sound: bool,
    ) -> Self {
        let mut app = Self {
            config,
            state,
            paths,
            prefs,
            focus: Pane::Activity,
            watched_selection: 0,
            activity_selection: 0,
            overlay: None,
            overlay_scroll: 0,
            status: String::new(),
            dirty: false,
            picker_selection: 0,
            settings_selection: 0,
            size: (80, 24),
            quit: false,
            selected_key: None,
            selected_activity_id: None,
            no_desktop,
            no_sound,
            core: PassuffCore::new(),
        };
        app.sync_selection();
        app
    }

    pub fn interactive(&self) -> bool {
        util::terminal_interactive()
    }

    // ----- read model -----------------------------------------------------

    pub fn watched_keys(&self) -> Vec<String> {
        self.watched_rows()
            .into_iter()
            .filter_map(|row| match row {
                WatchedRow::Pull(key) => Some(key),
                WatchedRow::StackHeader { .. } => None,
            })
            .collect()
    }

    pub fn watched_rows(&self) -> Vec<WatchedRow> {
        let login_set = self.state.login_set();
        let watched = self.state.watched_by_key();
        let mut standalones: Vec<WatchedPullRequest> = Vec::new();
        let mut stacks: BTreeMap<(String, u64), (StackInfo, Vec<WatchedPullRequest>)> =
            BTreeMap::new();

        for pull_request in watched.values() {
            if !view::scope_allows(&self.config, &pull_request.key) {
                continue;
            }
            if !view::watched_in_focus(&self.config, pull_request, &login_set) {
                continue;
            }
            match (
                &pull_request.stack,
                PullRequestKey::parse(&pull_request.key),
            ) {
                (Some(stack), Some(key)) => {
                    stacks
                        .entry((key.repository_name(), stack.number))
                        .or_insert_with(|| (stack.clone(), Vec::new()))
                        .1
                        .push((*pull_request).clone());
                }
                _ => standalones.push((*pull_request).clone()),
            }
        }

        #[allow(clippy::large_enum_variant)]
        enum Unit {
            Standalone(WatchedPullRequest),
            Stack(StackInfo, Vec<WatchedPullRequest>),
        }
        impl Unit {
            fn representative(&self) -> &WatchedPullRequest {
                match self {
                    Unit::Standalone(pull_request) => pull_request,
                    Unit::Stack(_, members) => &members[0],
                }
            }
        }

        let mut units: Vec<Unit> = standalones.into_iter().map(Unit::Standalone).collect();
        for (_, (stack, mut members)) in stacks {
            members.sort_by_key(|member| {
                member
                    .stack
                    .as_ref()
                    .map(|stack| stack.position)
                    .unwrap_or(0)
            });
            units.push(Unit::Stack(stack, members));
        }

        let sort = self.prefs.sort;
        units.sort_by(|left, right| {
            let left_key = left.representative();
            let right_key = right.representative();
            let ordering = match sort {
                SortMode::Updated => left_key
                    .updated_at
                    .cmp(&right_key.updated_at)
                    .then_with(|| natural_key_cmp(&left_key.key, &right_key.key)),
                SortMode::Name => left_key.key.cmp(&right_key.key),
                SortMode::State => state_rank(left_key.state)
                    .cmp(&state_rank(right_key.state))
                    .then_with(|| natural_key_cmp(&left_key.key, &right_key.key)),
                SortMode::Age => left_key
                    .created_at
                    .cmp(&right_key.created_at)
                    .then_with(|| natural_key_cmp(&left_key.key, &right_key.key)),
            };
            if self.prefs.ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        let mut rows = Vec::new();
        for unit in units {
            match unit {
                Unit::Standalone(pull_request) => rows.push(WatchedRow::Pull(pull_request.key)),
                Unit::Stack(stack, members) => {
                    rows.push(WatchedRow::StackHeader {
                        number: stack.number,
                        base: stack.base.clone(),
                    });
                    for member in members {
                        rows.push(WatchedRow::Pull(member.key));
                    }
                }
            }
        }
        rows
    }

    pub fn selected_watched_key(&self) -> Option<String> {
        self.watched_keys().get(self.watched_selection).cloned()
    }

    pub fn activity_base(&self) -> Vec<&ActivityRecord> {
        let watched = self.state.watched_by_key();
        let login_set = self.state.login_set();
        let mut records: Vec<&ActivityRecord> = self
            .state
            .activity
            .iter()
            .filter(|record| {
                if !view::scope_allows(&self.config, &record.key) {
                    return false;
                }
                if view::is_ignored(&self.config, &record.key) && !self.prefs.show_ignored {
                    return false;
                }
                match watched.get(&record.key) {
                    Some(pull_request) => {
                        view::watched_in_focus(&self.config, pull_request, &login_set)
                    }
                    None => true,
                }
            })
            .collect();
        records.sort_by(|left, right| {
            right
                .shown_time()
                .cmp(&left.shown_time())
                .then_with(|| right.observed_at.cmp(&left.observed_at))
                .then_with(|| activity_number(&right.id).cmp(&activity_number(&left.id)))
        });
        records
    }

    pub fn activity_filtered(&self) -> Vec<&ActivityRecord> {
        let base = self.activity_base();
        let watched = self.state.watched_by_key();
        base.into_iter()
            .filter(|record| {
                if self.prefs.unread_only && record.read {
                    return false;
                }
                match &self.prefs.filter {
                    None => true,
                    Some(filter) => match filter.as_str() {
                        _ if filter.starts_with("stack:") => {
                            let parts: Vec<&str> = filter.splitn(3, ':').collect();
                            if parts.len() != 3 {
                                return true;
                            }
                            let repository = parts[1];
                            let number: u64 = parts[2].parse().unwrap_or(0);
                            record_stack(&watched, record)
                                .map(|stack| {
                                    stack.number == number
                                        && PullRequestKey::parse(&record.key)
                                            .map(|key| {
                                                key.repository_name()
                                                    .eq_ignore_ascii_case(repository)
                                            })
                                            .unwrap_or(false)
                                })
                                .unwrap_or(false)
                        }
                        key => record.key == key,
                    },
                }
            })
            .collect()
    }

    pub fn activity_counts(&self) -> (usize, usize) {
        let records = self.activity_filtered();
        let unread = records.iter().filter(|record| !record.read).count();
        (unread, records.len())
    }

    pub fn global_counts(&self) -> (usize, usize) {
        let records = self.activity_base();
        let unread = records.iter().filter(|record| !record.read).count();
        (unread, records.len())
    }

    pub fn filtered_from(&self) -> Option<usize> {
        let base = self.activity_base().len();
        let filtered = self.activity_filtered().len();
        (base != filtered).then_some(base)
    }

    pub fn selected_activity(&self) -> Option<&ActivityRecord> {
        self.activity_filtered()
            .get(self.activity_selection)
            .copied()
    }

    pub fn activity_pane_title(&self) -> String {
        let mut parts = Vec::new();
        if let Some(filter) = &self.prefs.filter {
            if let Some(rest) = filter.strip_prefix("stack:") {
                let mut split = rest.splitn(2, ':');
                let repository = split.next().unwrap_or_default();
                let number = split.next().unwrap_or_default();
                parts.push(format!("{repository} stack {number}"));
            } else {
                parts.push(filter.clone());
            }
        }
        if self.prefs.unread_only {
            parts.push("unread".to_string());
        }
        if parts.is_empty() {
            "Activity · all".to_string()
        } else {
            format!("Activity · {}", parts.join(" · "))
        }
    }

    pub fn sync_selection(&mut self) {
        let keys = self.watched_keys();
        if keys.is_empty() {
            self.watched_selection = 0;
            self.selected_key = None;
        } else {
            if let Some(selected) = &self.selected_key {
                if let Some(index) = keys.iter().position(|key| key == selected) {
                    self.watched_selection = index;
                } else {
                    self.watched_selection = self.watched_selection.min(keys.len() - 1);
                    self.selected_key = keys.get(self.watched_selection).cloned();
                }
            } else {
                self.watched_selection = self.watched_selection.min(keys.len() - 1);
                self.selected_key = keys.get(self.watched_selection).cloned();
            }
        }

        let ids: Vec<String> = self
            .activity_filtered()
            .into_iter()
            .map(|record| record.id.clone())
            .collect();
        if ids.is_empty() {
            self.activity_selection = 0;
            self.selected_activity_id = None;
        } else {
            if let Some(selected) = &self.selected_activity_id {
                if let Some(index) = ids.iter().position(|id| id == selected) {
                    self.activity_selection = index;
                } else {
                    self.activity_selection = self.activity_selection.min(ids.len() - 1);
                    self.selected_activity_id = ids.get(self.activity_selection).cloned();
                }
            } else {
                self.activity_selection = self.activity_selection.min(ids.len() - 1);
                self.selected_activity_id = ids.get(self.activity_selection).cloned();
            }
        }
    }

    pub fn selected_repository_key(&self) -> Option<PullRequestKey> {
        self.selected_watched_key()
            .and_then(|key| PullRequestKey::parse(&key))
    }

    // ----- movement -------------------------------------------------------

    fn move_watched(&mut self, delta: isize) {
        let keys = self.watched_keys();
        if keys.is_empty() {
            return;
        }
        let next = clamp_index(self.watched_selection, delta, keys.len());
        self.watched_selection = next;
        self.selected_key = keys.get(next).cloned();
    }

    fn move_activity(&mut self, delta: isize) {
        let ids: Vec<String> = self
            .activity_filtered()
            .into_iter()
            .map(|record| record.id.clone())
            .collect();
        if ids.is_empty() {
            return;
        }
        let next = clamp_index(self.activity_selection, delta, ids.len());
        self.activity_selection = next;
        self.selected_activity_id = ids.get(next).cloned();
    }

    fn home_watched(&mut self) {
        let keys = self.watched_keys();
        if let Some(key) = keys.first() {
            self.watched_selection = 0;
            self.selected_key = Some(key.clone());
        }
    }

    fn end_watched(&mut self) {
        let keys = self.watched_keys();
        if let Some(key) = keys.last() {
            self.watched_selection = keys.len() - 1;
            self.selected_key = Some(key.clone());
        }
    }

    fn home_activity(&mut self) {
        let ids: Vec<String> = self
            .activity_filtered()
            .into_iter()
            .map(|record| record.id.clone())
            .collect();
        if let Some(id) = ids.first() {
            self.activity_selection = 0;
            self.selected_activity_id = Some(id.clone());
        }
    }

    fn end_activity(&mut self) {
        let ids: Vec<String> = self
            .activity_filtered()
            .into_iter()
            .map(|record| record.id.clone())
            .collect();
        if let Some(id) = ids.last() {
            self.activity_selection = ids.len() - 1;
            self.selected_activity_id = Some(id.clone());
        }
    }

    // ----- actions --------------------------------------------------------

    pub fn mark_selected_read(&mut self) {
        let Some(id) = self.selected_activity().map(|record| record.id.clone()) else {
            return;
        };
        let mut changed = false;
        if let Some(record) = self.state.find_activity_mut(&id) {
            if !record.read {
                record.read = true;
                changed = true;
            }
        }
        if changed && !self.save_state() {
            if let Some(record) = self.state.find_activity_mut(&id) {
                record.read = false;
            }
        }
        self.sync_selection();
    }

    pub fn mark_visible_read(&mut self) {
        let ids: Vec<String> = self
            .activity_filtered()
            .into_iter()
            .map(|record| record.id.clone())
            .collect();
        let mut changed = false;
        let mut changed_ids = Vec::new();
        for id in ids {
            if let Some(record) = self.state.find_activity_mut(&id) {
                if !record.read {
                    record.read = true;
                    changed = true;
                    changed_ids.push(id);
                }
            }
        }
        if changed && !self.save_state() {
            for id in &changed_ids {
                if let Some(record) = self.state.find_activity_mut(id) {
                    record.read = false;
                }
            }
        }
        self.sync_selection();
    }

    fn open_selected_detail(&mut self) {
        match self.focus {
            Pane::Activity => {
                if let Some(record) = self.selected_activity() {
                    self.overlay = Some(Overlay::ActivityDetail {
                        id: record.id.clone(),
                    });
                    self.overlay_scroll = 0;
                }
            }
            Pane::PullRequests => {
                if let Some(key) = self.selected_watched_key() {
                    self.overlay = Some(Overlay::PullRequestDetail { key });
                    self.overlay_scroll = 0;
                }
            }
        }
    }

    fn toggle_ignore(&mut self) {
        let Some(key) = self.selected_watched_key() else {
            return;
        };
        let previous = self.config.clone();
        let displayed = &self.config.watch.ignored_pull_requests;
        if let Some(index) = displayed
            .iter()
            .position(|ignored| ignored.eq_ignore_ascii_case(&key))
        {
            self.config.watch.ignored_pull_requests.remove(index);
        } else {
            self.config.watch.ignored_pull_requests.push(key);
        }
        if let Err(error) = passuff_config::save(&self.paths.config, &self.config) {
            self.config = previous;
            self.status = format!("cannot write configuration: {error}");
        }
        self.sync_selection();
    }

    fn set_filter_from_selection(&mut self, stack: bool) {
        let Some(key) = self.selected_watched_key() else {
            return;
        };
        if stack {
            if let Some(pull_request) = self.state.watched_by_key().get(&key).copied() {
                if let (Some(metadata), Some(parsed)) =
                    (&pull_request.stack, PullRequestKey::parse(&key))
                {
                    self.prefs.filter = Some(format!(
                        "stack:{}:{}",
                        parsed.repository_name(),
                        metadata.number
                    ));
                    self.dirty = true;
                    self.sync_selection();
                    return;
                }
            }
        }
        self.prefs.filter = Some(key);
        self.dirty = true;
        self.sync_selection();
    }

    fn open_settings(&mut self) {
        self.overlay = Some(Overlay::Settings);
        self.settings_selection = 0;
    }

    fn open_picker(&mut self) {
        self.overlay = Some(Overlay::Picker);
        self.picker_selection = 0;
    }

    fn open_help(&mut self) {
        self.overlay = Some(Overlay::Help { pane: self.focus });
    }

    pub fn picker_entries(&self) -> Vec<PickerEntry> {
        let mut entries = Vec::new();
        for relation in Relation::ALL {
            entries.push(PickerEntry::Focus(relation));
        }
        for repository in view::known_repositories(&self.config, &self.state) {
            entries.push(PickerEntry::Repository(repository));
        }
        entries
    }

    fn toggle_picker_entry(&mut self) {
        let entries = self.picker_entries();
        let Some(entry) = entries.get(self.picker_selection).cloned() else {
            return;
        };
        let previous = self.config.clone();
        match entry {
            PickerEntry::Focus(relation) => {
                if let Some(index) = self
                    .config
                    .focus
                    .relations
                    .iter()
                    .position(|candidate| *candidate == relation)
                {
                    self.config.focus.relations.remove(index);
                } else {
                    self.config.focus.relations.push(relation);
                    self.config
                        .focus
                        .relations
                        .sort_by_key(|relation| match relation {
                            Relation::Authored => 0,
                            Relation::Assigned => 1,
                            Relation::ReviewRequested => 2,
                            Relation::TeamReviewRequested => 3,
                        });
                }
            }
            PickerEntry::Repository(repository) => {
                let selected = self
                    .config
                    .watch
                    .repositories
                    .iter()
                    .position(|candidate| candidate.eq_ignore_ascii_case(&repository));
                match selected {
                    Some(index) => {
                        if self.config.watch.effective_mode() == WatchMode::All {
                            self.status = "cannot deselect in all watch mode".to_string();
                            return;
                        }
                        self.config.watch.repositories.remove(index);
                    }
                    None => {
                        self.config.watch.repositories.push(repository.clone());
                        self.config.watch.mode = Some(WatchMode::Selected);
                        if self
                            .state
                            .all_watched()
                            .iter()
                            .any(|watched| watched.repository().eq_ignore_ascii_case(&repository))
                        {
                            self.state.catch_up.insert(repository.to_ascii_lowercase());
                            if let Err(error) = state::save(&self.paths.state, &self.state) {
                                self.status = format!("state error: {error}");
                            }
                        }
                    }
                }
            }
        }
        if let Err(error) = passuff_config::save(&self.paths.config, &self.config) {
            self.config = previous;
            self.status = format!("cannot write configuration: {error}");
        }
        self.sync_selection();
    }

    fn toggle_picker_mode(&mut self) {
        let previous = self.config.clone();
        self.config.watch.mode = Some(match self.config.watch.effective_mode() {
            WatchMode::All => WatchMode::Selected,
            WatchMode::Selected => WatchMode::All,
        });
        if let Err(error) = passuff_config::save(&self.paths.config, &self.config) {
            self.config = previous;
            self.status = format!("cannot write configuration: {error}");
        }
        self.sync_selection();
    }

    fn save_prefs(&mut self) -> bool {
        match self.prefs.save(&self.paths.ui()) {
            Ok(()) => true,
            Err(error) => {
                self.status = format!("cannot save preferences: {error}");
                false
            }
        }
    }

    fn save_state(&mut self) -> bool {
        match state::save(&self.paths.state, &self.state) {
            Ok(()) => true,
            Err(error) => {
                self.status = format!("state error: {error}");
                false
            }
        }
    }

    pub fn detail_url(&self) -> Option<String> {
        match &self.overlay {
            Some(Overlay::ActivityDetail { id }) => self
                .state
                .activity
                .iter()
                .find(|record| &record.id == id)
                .and_then(|record| PullRequestKey::parse(&record.key))
                .map(|key| key.web_url()),
            Some(Overlay::PullRequestDetail { key }) => {
                PullRequestKey::parse(key).map(|key| key.web_url())
            }
            _ => None,
        }
    }

    pub fn detail_lines(&self) -> Vec<String> {
        match &self.overlay {
            Some(Overlay::ActivityDetail { id }) => {
                let record = self.state.activity.iter().find(|record| &record.id == id);
                match record {
                    Some(record) => render::activity_detail_lines(self, record),
                    None => vec!["no activity".to_string()],
                }
            }
            Some(Overlay::PullRequestDetail { key }) => {
                let pull_request = self.state.watched_by_key().get(key).copied();
                match pull_request {
                    Some(pull_request) => render::pull_request_detail_lines(self, pull_request),
                    None => vec!["no pull request".to_string()],
                }
            }
            Some(Overlay::Help { pane }) => render::help_lines(*pane),
            Some(Overlay::Settings) => render::settings_lines(self),
            Some(Overlay::Picker) => render::picker_lines(self),
            Some(Overlay::SavePrompt) => vec![
                "Save view and notification settings?".to_string(),
                String::new(),
                "y save    n discard    esc/q return".to_string(),
            ],
            None => Vec::new(),
        }
    }

    pub fn overlay_title(&self) -> String {
        match &self.overlay {
            Some(Overlay::ActivityDetail { .. }) => "Activity".to_string(),
            Some(Overlay::PullRequestDetail { .. }) => "Pull request".to_string(),
            Some(Overlay::Help { .. }) => "Keys".to_string(),
            Some(Overlay::Settings) => "Settings".to_string(),
            Some(Overlay::Picker) => "Watch scope".to_string(),
            Some(Overlay::SavePrompt) => "Save".to_string(),
            None => String::new(),
        }
    }

    fn overlay_max_scroll(&self) -> usize {
        let inner_height = overlay_inner_height(self.size);
        self.detail_lines().len().saturating_sub(inner_height)
    }

    // ----- keys -----------------------------------------------------------

    pub fn handle_key(&mut self, key: Key) {
        if key == Key::CtrlC {
            self.quit = true;
            return;
        }
        if let Some(overlay) = self.overlay.clone() {
            self.handle_overlay_key(key, overlay);
            self.sync_selection();
            return;
        }

        match key {
            Key::Tab | Key::BackTab => {
                self.focus = match self.focus {
                    Pane::PullRequests => Pane::Activity,
                    Pane::Activity => Pane::PullRequests,
                };
            }
            Key::Up | Key::Char('k') => match self.focus {
                Pane::PullRequests => self.move_watched(-1),
                Pane::Activity => self.move_activity(-1),
            },
            Key::Down | Key::Char('j') => match self.focus {
                Pane::PullRequests => self.move_watched(1),
                Pane::Activity => self.move_activity(1),
            },
            Key::PageUp => match self.focus {
                Pane::PullRequests => self.move_watched(-10),
                Pane::Activity => self.move_activity(-10),
            },
            Key::PageDown => match self.focus {
                Pane::PullRequests => self.move_watched(10),
                Pane::Activity => self.move_activity(10),
            },
            Key::Home => match self.focus {
                Pane::PullRequests => self.home_watched(),
                Pane::Activity => self.home_activity(),
            },
            Key::End => match self.focus {
                Pane::PullRequests => self.end_watched(),
                Pane::Activity => self.end_activity(),
            },
            Key::Enter => self.open_selected_detail(),
            Key::Esc | Key::Char('q') => {
                if self.dirty {
                    self.overlay = Some(Overlay::SavePrompt);
                } else {
                    self.quit = true;
                }
            }
            Key::Char('r') => {
                if self.focus == Pane::Activity {
                    self.mark_selected_read();
                }
            }
            Key::Char('a') => {
                if self.focus == Pane::Activity {
                    self.mark_visible_read();
                }
            }
            Key::Char('u') => {
                self.prefs.unread_only = !self.prefs.unread_only;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char('c') => {
                self.prefs.filter = None;
                self.prefs.unread_only = false;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char('C') => {
                self.prefs.sort = SortMode::Updated;
                self.prefs.ascending = true;
                self.prefs.filter = None;
                self.prefs.unread_only = false;
                self.prefs.show_ignored = false;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char('s') => {
                self.prefs.sort = self.prefs.sort.next();
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char('S') => {
                self.prefs.ascending = !self.prefs.ascending;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char('f') => self.set_filter_from_selection(false),
            Key::Char('F') => self.set_filter_from_selection(true),
            Key::Char('x') => {
                if self.focus == Pane::PullRequests {
                    self.toggle_ignore();
                }
            }
            Key::Left => {
                self.prefs.show_ignored = false;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Right => {
                self.prefs.show_ignored = true;
                self.dirty = true;
                self.sync_selection();
            }
            Key::Char(',') => self.open_settings(),
            Key::Char('p') => self.open_picker(),
            Key::Char('h') | Key::Char('H') | Key::Function(1) => self.open_help(),
            _ => {}
        }
    }

    fn handle_overlay_key(&mut self, key: Key, overlay: Overlay) {
        match overlay {
            Overlay::ActivityDetail { .. } | Overlay::PullRequestDetail { .. } => match key {
                Key::Up | Key::Char('k') => {
                    self.overlay_scroll = self.overlay_scroll.saturating_sub(1);
                }
                Key::Down | Key::Char('j') => {
                    self.overlay_scroll = (self.overlay_scroll + 1).min(self.overlay_max_scroll());
                }
                Key::PageUp => {
                    self.overlay_scroll = self.overlay_scroll.saturating_sub(10);
                }
                Key::PageDown => {
                    self.overlay_scroll = (self.overlay_scroll + 10).min(self.overlay_max_scroll());
                }
                Key::Home => self.overlay_scroll = 0,
                Key::End => self.overlay_scroll = self.overlay_max_scroll(),
                Key::Enter => {
                    if let Some(url) = self.detail_url() {
                        match browser::open(&url) {
                            Ok(()) => self.overlay = None,
                            Err(error) => self.status = error,
                        }
                    }
                }
                Key::Esc | Key::Char('q') => self.overlay = None,
                _ => {}
            },
            Overlay::Help { .. } => {
                if matches!(
                    key,
                    Key::Esc | Key::Char('q') | Key::Char('h') | Key::Char('H') | Key::Function(1)
                ) {
                    self.overlay = None;
                }
            }
            Overlay::Settings => match key {
                Key::Up | Key::Char('k') => {
                    self.settings_selection = self.settings_selection.saturating_sub(1);
                }
                Key::Down | Key::Char('j') => {
                    self.settings_selection = (self.settings_selection + 1).min(1);
                }
                Key::Char(' ') | Key::Enter => {
                    if self.settings_selection == 0 {
                        self.prefs.desktop = !self.prefs.desktop;
                    } else {
                        self.prefs.sound = !self.prefs.sound;
                    }
                    self.dirty = true;
                }
                Key::Esc | Key::Char(',') | Key::Char('q') => self.overlay = None,
                _ => {}
            },
            Overlay::Picker => {
                let entries = self.picker_entries();
                match key {
                    Key::Up | Key::Char('k') => {
                        self.picker_selection = self.picker_selection.saturating_sub(1);
                    }
                    Key::Down | Key::Char('j') => {
                        if !entries.is_empty() {
                            self.picker_selection =
                                (self.picker_selection + 1).min(entries.len() - 1);
                        }
                    }
                    Key::Char(' ') => self.toggle_picker_entry(),
                    Key::Char('m') => self.toggle_picker_mode(),
                    Key::Esc | Key::Char('q') => self.overlay = None,
                    _ => {}
                }
            }
            Overlay::SavePrompt => match key {
                Key::Char('y') => {
                    if self.save_prefs() {
                        self.quit = true;
                    }
                }
                Key::Char('n') => self.quit = true,
                Key::Esc | Key::Char('q') => self.overlay = None,
                _ => {}
            },
        }
    }

    // ----- observation ----------------------------------------------------

    fn reconciliation(&mut self, interactive: bool, deliver: bool) {
        let report = self
            .core
            .refresh_targeted(&self.config, &mut self.state, interactive);
        self.after_observation(report, deliver);
    }

    fn probe_once(&mut self, interactive: bool, deliver: bool) -> Duration {
        let report = self.core.probe(&self.config, &mut self.state, interactive);
        let mut status = report
            .failures
            .first()
            .map(|failure| format!("account {} failed: {}", failure.label, failure.message))
            .unwrap_or_default();
        let delay = probe_delay(&self.config, &report.rate_limits);
        if !report.changed.is_empty() {
            let changed_report = self.core.observe_changed(
                &self.config,
                &mut self.state,
                &report.changed,
                &report.logins,
                interactive,
            );
            let failure = changed_report
                .failures
                .first()
                .map(|failure| format!("account {} failed: {}", failure.label, failure.message));
            if failure.is_some() {
                status = failure.unwrap_or_default();
            }
            self.after_observation(changed_report, deliver);
        } else if report.failures.is_empty() && report.logins.is_empty() {
            // Nothing observed and nothing failed (no configured accounts is
            // already a configuration error).
        }
        if !status.is_empty() {
            self.status = status;
        }
        self.sync_selection();
        delay
    }

    fn after_observation(&mut self, report: RefreshReport, deliver: bool) {
        if let Err(error) = state::save(&self.paths.state, &self.state) {
            self.status = format!("state error: {error}");
        }
        if let Some(failure) = report.failures.first() {
            self.status = format!("account {} failed: {}", failure.label, failure.message);
        }
        if deliver {
            self.deliver_report(&report);
        }
        self.sync_selection();
    }

    fn deliver_report(&mut self, report: &RefreshReport) {
        let Some(digest) = cli_render::digest(&report.notifications) else {
            return;
        };
        let surfaces = delivery::Surfaces {
            desktop: self.prefs.desktop && !self.no_desktop,
            sound: self.prefs.sound && !self.no_sound,
            urgency: self.config.notifications.urgency,
            sound_file: self.config.notifications.sound_file.clone(),
        };
        let result = delivery::deliver(&digest, &surfaces);
        if let Some(error) = result.desktop_error {
            self.status = format!("desktop notification failed: {error}");
        }
        if let Some(error) = result.sound_error {
            self.status = error;
        }
    }

    fn reload_config(&mut self) {
        match passuff_config::load(&self.paths.config) {
            Ok(config) => self.config = config,
            Err(error) => self.status = format!("configuration error: {error}"),
        }
        self.sync_selection();
    }
}

#[derive(Debug, Clone)]
pub enum PickerEntry {
    Focus(Relation),
    Repository(String),
}

pub fn main() -> i32 {
    let cli = TuiCli::parse();
    let paths = Paths::resolve(cli.config.clone(), cli.state.clone());

    let mut config = match load_or_setup_config(&paths) {
        Ok(config) => config,
        Err(code) => return code,
    };

    let mut loaded = match state::load(&paths.state) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("passuff-tui: state error: {error}");
            return 3;
        }
    };
    if loaded.needs_save {
        if let Err(error) = state::save(&paths.state, &loaded.state) {
            eprintln!("passuff-tui: state error: {error}");
            return 3;
        }
    }

    if let Err(code) = apply_scope_flags(&cli, &paths, &mut config, &mut loaded.state) {
        return code;
    }

    let has_terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let observing_requested =
        cli.observe || cli.observe_for.is_some() || (!cli.snapshot && has_terminal);

    if cli.snapshot {
        if observing_requested {
            if let Err(code) =
                observe_before_open(&cli, &paths, &mut config, &mut loaded.state, !has_terminal)
            {
                return code;
            }
        }
        let prefs = UiPrefs::load(&paths.ui(), &config);
        let mut app = App::new(
            config,
            loaded.state,
            paths.clone(),
            prefs,
            cli.no_desktop,
            cli.no_sound,
        );
        app.size = snapshot_size();
        if cli.picker {
            app.overlay = Some(Overlay::Picker);
        }
        if let Some(keys) = &cli.keys {
            match parse_keys(keys) {
                Ok(keys) => {
                    for key in keys {
                        app.handle_key(key);
                    }
                }
                Err(message) => {
                    eprintln!("passuff-tui: {message}");
                    return 2;
                }
            }
        }
        println!("{}", render::frame(&app).join("\n"));
        return 0;
    }

    if !has_terminal {
        eprintln!("passuff-tui: requires a terminal; use --snapshot to print one frame");
        return 3;
    }

    let explicit_observe = cli.observe || cli.observe_for.is_some();
    if explicit_observe {
        if let Err(code) = observe_before_open(&cli, &paths, &mut config, &mut loaded.state, false)
        {
            return code;
        }
    }

    let prefs = UiPrefs::load(&paths.ui(), &config);
    let mut app = App::new(
        config,
        loaded.state,
        paths.clone(),
        prefs,
        cli.no_desktop,
        cli.no_sound,
    );
    if cli.picker {
        app.overlay = Some(Overlay::Picker);
    }
    if let Some(keys) = &cli.keys {
        match parse_keys(keys) {
            Ok(keys) => {
                for key in keys {
                    app.handle_key(key);
                }
            }
            Err(message) => {
                eprintln!("passuff-tui: {message}");
                return 2;
            }
        }
    }
    app.run_live(!explicit_observe)
}

fn load_or_setup_config(paths: &Paths) -> Result<AppConfig, i32> {
    match passuff_config::load(&paths.config) {
        Ok(config) => Ok(config),
        Err(passuff_config::LoadError::Missing(_)) => {
            if !util::terminal_interactive() {
                eprintln!("passuff-tui: no configuration; run passuff setup");
                return Err(2);
            }
            match setup::run(&paths.config) {
                Ok(_) => match passuff_config::load(&paths.config) {
                    Ok(config) => Ok(config),
                    Err(error) => {
                        eprintln!("passuff-tui: configuration error: {error}");
                        Err(2)
                    }
                },
                Err(setup::SetupError::Validation(message)) => {
                    println!("passuff-tui: {message}");
                    Err(2)
                }
                Err(setup::SetupError::Cancelled) => {
                    println!("passuff-tui: setup cancelled");
                    Err(2)
                }
                Err(setup::SetupError::Io(message)) => {
                    println!("passuff-tui: cannot write configuration: {message}");
                    Err(3)
                }
            }
        }
        Err(error) => {
            eprintln!("passuff-tui: configuration error: {error}");
            Err(2)
        }
    }
}

fn apply_scope_flags(
    cli: &TuiCli,
    paths: &Paths,
    config: &mut AppConfig,
    state: &mut WatcherState,
) -> Result<(), i32> {
    let previous = config.clone();
    let mut changed = false;

    if let Some(mode) = &cli.scope_mode {
        match mode.as_str() {
            "all" => {
                config.watch.mode = Some(WatchMode::All);
                changed = true;
            }
            "selected" => {
                config.watch.mode = Some(WatchMode::Selected);
                changed = true;
            }
            other => {
                eprintln!("passuff-tui: unknown watch mode: {other}");
                return Err(2);
            }
        }
    }

    for repository in &cli.scope_select {
        if passuff_config::validate_repository(repository).is_err() {
            eprintln!("passuff-tui: repository must be owner/repo: {repository}");
            return Err(2);
        }
        if !config
            .watch
            .repositories
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(repository))
        {
            config.watch.repositories.push(repository.clone());
            if state
                .all_watched()
                .iter()
                .any(|watched| watched.repository().eq_ignore_ascii_case(repository))
            {
                state.catch_up.insert(repository.to_ascii_lowercase());
            }
        }
        config.watch.mode = Some(WatchMode::Selected);
        changed = true;
    }

    for repository in &cli.scope_deselect {
        if config.watch.effective_mode() == WatchMode::All {
            eprintln!("passuff-tui: cannot deselect {repository} in all watch mode");
            return Err(2);
        }
        config
            .watch
            .repositories
            .retain(|candidate| !candidate.eq_ignore_ascii_case(repository));
        changed = true;
    }

    if let Some(focus) = &cli.focus_set {
        let mut relations = Vec::new();
        for raw in focus.split(',') {
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            match Relation::parse(raw) {
                Some(relation) => relations.push(relation),
                None => {
                    eprintln!("passuff-tui: unknown focus relation: {raw}");
                    return Err(2);
                }
            }
        }
        config.focus.relations = relations;
        changed = true;
    }

    if changed {
        if let Err(error) = passuff_config::save(&paths.config, config) {
            *config = previous;
            eprintln!("passuff-tui: cannot write configuration: {error}");
            return Err(2);
        }
        if let Err(error) = state::save(&paths.state, state) {
            eprintln!("passuff-tui: state error: {error}");
            return Err(3);
        }
    }
    Ok(())
}

fn observe_before_open(
    cli: &TuiCli,
    paths: &Paths,
    config: &mut AppConfig,
    state: &mut WatcherState,
    _snapshot_without_terminal: bool,
) -> Result<(), i32> {
    let mut core = PassuffCore::new();
    let interactive = util::terminal_interactive();
    let _report = core.refresh_targeted(config, state, interactive);
    if let Err(error) = state::save(&paths.state, state) {
        eprintln!("passuff-tui: state error: {error}");
        return Err(3);
    }
    if cli.observe_for.is_none() {
        return Ok(());
    }
    let duration = Duration::from_millis(cli.observe_for.unwrap_or(0));
    let started = Instant::now();
    let mut next_probe = Instant::now();
    while started.elapsed() < duration {
        if Instant::now() >= next_probe {
            let probe = core.probe(config, state, interactive);
            if !probe.changed.is_empty() {
                let _ =
                    core.observe_changed(config, state, &probe.changed, &probe.logins, interactive);
                let _ = state::save(&paths.state, state);
            }
            next_probe = Instant::now() + probe_delay(config, &probe.rate_limits);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

impl App {
    fn run_live(&mut self, observe_startup: bool) -> i32 {
        let _ = ctrlc::set_handler(|| {
            TUI_INTERRUPTED.store(true, Ordering::SeqCst);
        });
        TUI_INTERRUPTED.store(false, Ordering::SeqCst);
        if let Err(error) = terminal::enable_raw_mode() {
            eprintln!("passuff-tui: cannot enter raw mode: {error}");
            return 3;
        }
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide);
        let guard = TerminalGuard;
        let interactive = true;

        // A live inbox observes the configured accounts itself.
        if observe_startup {
            self.reconciliation(interactive, true);
        }

        let reconciliation_interval =
            Duration::from_secs(self.config.reconciliation_interval_secs.max(1));
        let mut next_reconcile = Instant::now() + reconciliation_interval;
        let mut next_probe =
            Instant::now() + Duration::from_secs(self.config.poll_interval_secs.max(1));

        let initial = render::frame(self);
        draw_frame(&initial);

        loop {
            if self.quit || TUI_INTERRUPTED.load(Ordering::SeqCst) {
                break;
            }
            if let Ok((width, height)) = terminal::size() {
                self.size = (width, height);
            }
            let wait = next_probe
                .min(next_reconcile)
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(100));
            let mut redraw = false;
            match event::poll(wait) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(event)) => {
                        if event.kind == KeyEventKind::Press || event.kind == KeyEventKind::Repeat {
                            self.handle_key(map_key(event));
                            redraw = true;
                        }
                    }
                    Ok(Event::Resize(width, height)) => {
                        self.size = (width, height);
                        redraw = true;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        self.status = format!("terminal input error: {error}");
                        break;
                    }
                },
                Ok(false) => {}
                Err(error) => {
                    self.status = format!("terminal input error: {error}");
                    break;
                }
            }

            if TUI_INTERRUPTED.load(Ordering::SeqCst) {
                break;
            }

            let now = Instant::now();
            if now >= next_reconcile {
                self.reload_config();
                self.reconciliation(interactive, true);
                next_reconcile =
                    now + Duration::from_secs(self.config.reconciliation_interval_secs.max(1));
                next_probe = now + Duration::from_secs(self.config.poll_interval_secs.max(1));
                redraw = true;
            } else if now >= next_probe {
                self.reload_config();
                let delay = self.probe_once(interactive, true);
                next_probe = now + delay;
                redraw = true;
            }

            if redraw {
                let frame = render::frame(self);
                draw_frame(&frame);
            }
        }
        drop(guard);
        0
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

fn draw_frame(lines: &[String]) {
    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "\x1b[2J\x1b[H");
    for (index, line) in lines.iter().enumerate() {
        let _ = write!(stdout, "{line}");
        if index + 1 < lines.len() {
            let _ = write!(stdout, "\r\n");
        }
    }
    let _ = stdout.flush();
}

fn snapshot_size() -> (u16, u16) {
    if std::io::stdout().is_terminal() {
        if let Ok(size) = terminal::size() {
            return size;
        }
    }
    if let (Ok(columns), Ok(lines)) = (std::env::var("COLUMNS"), std::env::var("LINES")) {
        if let (Ok(columns), Ok(lines)) = (columns.parse(), lines.parse()) {
            return (columns, lines);
        }
    }
    (80, 24)
}

fn probe_delay(config: &AppConfig, rate_limits: &[passuff_core::github::RateLimit]) -> Duration {
    let mut delay = Duration::from_secs(config.poll_interval_secs.max(1));
    let now = Utc::now();
    for limit in rate_limits {
        let Some(remaining) = limit.remaining else {
            continue;
        };
        if remaining == 0 {
            if let Some(reset) = limit.reset_at.as_deref().and_then(parse_time) {
                let seconds = (reset - now).num_seconds().max(1) as u64;
                delay = delay.max(Duration::from_secs(seconds));
            }
            continue;
        }
        if remaining >= 5_000 {
            continue;
        }
        if let Some(reset) = limit.reset_at.as_deref().and_then(parse_time) {
            let seconds = (reset - now).num_seconds().max(0) as u64;
            let spacing = seconds.div_ceil(remaining.max(1)).max(1);
            delay = delay.max(Duration::from_secs(spacing));
        }
    }
    delay
}

fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|time| time.with_timezone(&Utc))
}

fn record_stack<'a>(
    watched: &'a BTreeMap<String, &'a WatchedPullRequest>,
    record: &ActivityRecord,
) -> Option<StackInfo> {
    record.stack.clone().or_else(|| {
        watched
            .get(&record.key)
            .and_then(|pull_request| pull_request.stack.clone())
    })
}

fn natural_key_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    match (PullRequestKey::parse(left), PullRequestKey::parse(right)) {
        (Some(left), Some(right)) => left
            .repository_name()
            .to_ascii_lowercase()
            .cmp(&right.repository_name().to_ascii_lowercase())
            .then_with(|| left.number.cmp(&right.number)),
        _ => left.cmp(right),
    }
}

fn state_rank(state: passuff_core::PrState) -> u8 {
    match state {
        passuff_core::PrState::Open => 0,
        passuff_core::PrState::Merged => 1,
        passuff_core::PrState::Closed => 2,
    }
}

fn clamp_index(current: usize, delta: isize, length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    let candidate = current as isize + delta;
    candidate.clamp(0, length as isize - 1) as usize
}

fn activity_number(id: &str) -> u64 {
    id.strip_prefix('a')
        .and_then(|number| number.parse().ok())
        .unwrap_or(0)
}

pub fn overlay_size(size: (u16, u16)) -> (u16, u16) {
    let width = (size.0.saturating_sub(4)).clamp(20, 100);
    let height = (size.1.saturating_sub(2)).clamp(6, 30);
    (width, height)
}

pub fn overlay_inner_height(size: (u16, u16)) -> usize {
    overlay_size(size).1.saturating_sub(2) as usize
}

fn map_key(event: KeyEvent) -> Key {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char('c' | 'C') = event.code {
            return Key::CtrlC;
        }
    }
    match event.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::F(number) => Key::Function(number),
        KeyCode::Char(character) => Key::Char(character),
        _ => Key::Char('\0'),
    }
}

pub fn parse_keys(value: &str) -> Result<Vec<Key>, String> {
    let mut keys = Vec::new();
    for token in value.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        keys.push(match token.to_ascii_lowercase().as_str() {
            "tab" => Key::Tab,
            "backtab" | "shift-tab" => Key::BackTab,
            "enter" => Key::Enter,
            "esc" | "escape" => Key::Esc,
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "home" => Key::Home,
            "end" => Key::End,
            "space" => Key::Char(' '),
            "comma" => Key::Char(','),
            single if single.chars().count() == 1 => {
                Key::Char(single.chars().next().unwrap_or('\0'))
            }
            other => return Err(format!("unknown key: {other}")),
        });
    }
    Ok(keys)
}
