use std::cell::Cell;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use passuff_config::{AppConfig, FocusRelation, ScopeEditor, WatchMode};
use passuff_core::{
    ActivityView, InboxService, InboxView, PullRequestKey, PullRequestView, SnapshotStore,
    StackInfo,
};
use serde::{Deserialize, Serialize};

use crate::runtime::BrowserOpener;
use crate::tui::keys::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Watched,
    Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    Updated,
    Name,
    State,
    Age,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Name => "name",
            Self::State => "state",
            Self::Age => "age",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Updated => Self::Name,
            Self::Name => Self::State,
            Self::State => Self::Age,
            Self::Age => Self::Updated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn arrow(self) -> &'static str {
        match self {
            Self::Asc => "↑",
            Self::Desc => "↓",
        }
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiPrefs {
    #[serde(default)]
    pub desktop: Option<bool>,
    #[serde(default)]
    pub sound: Option<bool>,
    #[serde(default)]
    pub sort: Option<SortMode>,
    #[serde(default)]
    pub direction: Option<SortDirection>,
    #[serde(default)]
    pub filter_pull_request: Option<String>,
    #[serde(default)]
    pub filter_stack: Option<u64>,
    #[serde(default)]
    pub unread_only: Option<bool>,
    #[serde(default)]
    pub show_ignored: Option<bool>,
}

impl UiPrefs {
    pub fn desktop(&self) -> bool {
        self.desktop.unwrap_or(true)
    }

    pub fn sound(&self) -> bool {
        self.sound.unwrap_or(true)
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Overlay {
    ActivityDetail {
        activity: ActivityView,
        pull_request: Option<PullRequestView>,
        scroll: usize,
    },
    PullRequestDetail {
        pull_request: PullRequestView,
        scroll: usize,
    },
    Help,
    Settings {
        selected: usize,
    },
    ScopePicker {
        selected: usize,
    },
    SavePrompt,
}

#[derive(Debug, Clone)]
pub enum WatchedRow {
    StackHeader {
        repository: String,
        stack: StackInfo,
    },
    PullRequest(PullRequestView),
}

pub struct TuiApp {
    pub config: AppConfig,
    pub config_path: PathBuf,
    pub ui_path: PathBuf,
    pub store: Arc<dyn SnapshotStore>,
    pub view: InboxView,
    pub prefs: UiPrefs,
    pub focus: Pane,
    pub watched_selection: usize,
    pub activity_selection: usize,
    pub sort: SortMode,
    pub direction: SortDirection,
    pub filter_pull_request: Option<PullRequestKey>,
    pub filter_stack: Option<u64>,
    pub unread_only: bool,
    pub show_ignored: bool,
    pub overlay: Option<Overlay>,
    pub status: Option<String>,
    pub browser: Arc<dyn BrowserOpener>,
    pub should_quit: bool,
    pub dirty: bool,
    pub interactive_mode: bool,
    pub detail_viewport: Cell<usize>,
}

impl TuiApp {
    pub fn new(
        config: AppConfig,
        config_path: PathBuf,
        ui_path: PathBuf,
        store: Arc<dyn SnapshotStore>,
        view: InboxView,
        prefs: UiPrefs,
        browser: Arc<dyn BrowserOpener>,
    ) -> Self {
        let sort = prefs.sort.unwrap_or(SortMode::Updated);
        let direction = prefs.direction.unwrap_or(SortDirection::Asc);
        let filter_pull_request = prefs
            .filter_pull_request
            .as_deref()
            .and_then(|value| value.parse::<PullRequestKey>().ok());
        let filter_stack = prefs.filter_stack;
        let unread_only = prefs.unread_only.unwrap_or(false);
        let show_ignored = prefs.show_ignored.unwrap_or(true);
        Self {
            config,
            config_path,
            ui_path,
            store,
            view,
            prefs,
            focus: Pane::Activity,
            watched_selection: 0,
            activity_selection: 0,
            sort,
            direction,
            filter_pull_request,
            filter_stack,
            unread_only,
            show_ignored,
            overlay: None,
            status: None,
            browser,
            should_quit: false,
            dirty: false,
            interactive_mode: false,
            detail_viewport: Cell::new(20),
        }
    }

    pub fn reload_view(&mut self) -> Result<(), String> {
        let selected_activity = self
            .visible_activity()
            .get(self.activity_selection)
            .map(|activity| activity.id.clone());
        let selected_pr = self.selected_pull_request_owned().map(|pr| pr.key.clone());
        let service = InboxService::new(self.config.clone(), self.store.clone());
        self.view = service.view().map_err(|error| error.to_string())?;
        self.watched_selection = selected_pr
            .and_then(|key| {
                self.sorted_pull_requests()
                    .iter()
                    .position(|pr| pr.key == key)
            })
            .unwrap_or(0);
        self.activity_selection = selected_activity
            .and_then(|id| {
                self.visible_activity()
                    .iter()
                    .position(|activity| activity.id == id)
            })
            .unwrap_or(0);
        self.clamp_selections();
        Ok(())
    }

    pub fn sorted_pull_requests(&self) -> Vec<PullRequestView> {
        let mut groups: std::collections::BTreeMap<(String, u64), Vec<PullRequestView>> =
            std::collections::BTreeMap::new();
        let mut standalone = Vec::new();
        for pull_request in self.view.watched.clone() {
            if let Some(stack) = &pull_request.stack {
                groups
                    .entry((pull_request.key.repository().to_string(), stack.number))
                    .or_default()
                    .push(pull_request);
            } else {
                standalone.push(pull_request);
            }
        }
        let mut units: Vec<Vec<PullRequestView>> = Vec::new();
        for (_, mut members) in groups {
            members.sort_by_key(|pr| pr.stack.as_ref().map(|stack| stack.position).unwrap_or(0));
            units.push(members);
        }
        for pull_request in standalone {
            units.push(vec![pull_request]);
        }
        units.sort_by(|left, right| {
            compare_units(left.first(), right.first(), self.sort, self.direction)
        });
        units.into_iter().flatten().collect()
    }

    pub fn watched_rows(&self) -> Vec<WatchedRow> {
        let mut rows = Vec::new();
        let mut seen_stacks: BTreeSet<(String, u64)> = BTreeSet::new();
        for pull_request in self.sorted_pull_requests() {
            if let Some(stack) = &pull_request.stack {
                let key = (pull_request.key.repository().to_string(), stack.number);
                if seen_stacks.insert(key.clone()) {
                    rows.push(WatchedRow::StackHeader {
                        repository: pull_request.key.repository().to_string(),
                        stack: stack.clone(),
                    });
                }
            }
            rows.push(WatchedRow::PullRequest(pull_request));
        }
        rows
    }

    pub fn selected_pull_request_owned(&self) -> Option<PullRequestView> {
        self.sorted_pull_requests()
            .get(self.watched_selection)
            .cloned()
    }

    pub fn visible_activity(&self) -> Vec<ActivityView> {
        self.view
            .activity
            .iter()
            .filter(|activity| self.show_ignored || !activity.ignored)
            .filter(|activity| {
                if let Some(key) = &self.filter_pull_request {
                    if &activity.pull_request != key {
                        return false;
                    }
                }
                if let Some(stack_number) = self.filter_stack {
                    let Some(stack) = &activity.stack else {
                        return false;
                    };
                    if stack.number != stack_number {
                        return false;
                    }
                }
                if self.unread_only && activity.read {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn visible_counts(&self) -> (usize, usize) {
        let visible = self.visible_activity();
        let unread = visible.iter().filter(|activity| !activity.read).count();
        (unread, visible.len())
    }

    pub fn filtered_from(&self) -> Option<usize> {
        if self.filter_pull_request.is_some() || self.filter_stack.is_some() || self.unread_only {
            Some(self.view.activity.len())
        } else {
            None
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        if self.overlay.is_some() {
            self.handle_overlay_key(key);
            return;
        }
        match key {
            Key::Tab => {
                self.focus = match self.focus {
                    Pane::Activity => Pane::Watched,
                    Pane::Watched => Pane::Activity,
                };
            }
            Key::BackTab => {
                self.focus = match self.focus {
                    Pane::Activity => Pane::Watched,
                    Pane::Watched => Pane::Activity,
                };
            }
            Key::Down => self.move_selection(1),
            Key::Up => self.move_selection(-1),
            Key::PageDown => self.move_selection(10),
            Key::PageUp => self.move_selection(-10),
            Key::Home => self.jump_selection(true),
            Key::End => self.jump_selection(false),
            Key::Enter => self.open_selected(),
            Key::Esc | Key::Quit => {
                if self.interactive_mode && self.dirty {
                    self.overlay = Some(Overlay::SavePrompt);
                } else {
                    self.should_quit = true;
                }
            }
            Key::MarkRead => self.mark_selected_read(),
            Key::MarkAllRead => self.mark_visible_read(),
            Key::ToggleUnread => {
                self.unread_only = !self.unread_only;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::ClearFilter => {
                self.filter_pull_request = None;
                self.filter_stack = None;
                self.unread_only = false;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::ResetView => {
                self.sort = SortMode::Updated;
                self.direction = SortDirection::Asc;
                self.filter_pull_request = None;
                self.filter_stack = None;
                self.unread_only = false;
                self.show_ignored = true;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::SortNext => {
                self.sort = self.sort.next();
                self.direction = SortDirection::Asc;
                self.dirty = true;
            }
            Key::SortReverse => {
                self.direction = self.direction.reverse();
                self.dirty = true;
            }
            Key::FilterPullRequest => {
                if let Some(pr) = self.selected_pull_request_owned() {
                    self.filter_pull_request = Some(pr.key);
                    self.filter_stack = None;
                    self.dirty = true;
                    self.clamp_selections();
                }
            }
            Key::FilterStack => {
                if let Some(pr) = self.selected_pull_request_owned() {
                    if let Some(stack) = pr.stack {
                        self.filter_stack = Some(stack.number);
                        self.filter_pull_request = None;
                    } else {
                        self.filter_pull_request = Some(pr.key);
                        self.filter_stack = None;
                    }
                    self.dirty = true;
                    self.clamp_selections();
                }
            }
            Key::ToggleIgnore => self.toggle_ignore(),
            Key::ToggleIgnoredVisibility => {
                self.show_ignored = !self.show_ignored;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::Left => {
                self.show_ignored = false;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::Right => {
                self.show_ignored = true;
                self.dirty = true;
                self.clamp_selections();
            }
            Key::OpenSettings => self.overlay = Some(Overlay::Settings { selected: 0 }),
            Key::OpenPicker => {
                let selected = self.known_repositories().len();
                self.overlay = Some(Overlay::ScopePicker { selected });
            }
            Key::Help => self.overlay = Some(Overlay::Help),
            Key::ToggleMode => {}
            Key::Space => {}
            Key::Char(character) => match character {
                ',' => self.overlay = Some(Overlay::Settings { selected: 0 }),
                'p' => {
                    let selected = self.known_repositories().len();
                    self.overlay = Some(Overlay::ScopePicker { selected });
                }
                'h' | 'H' => self.overlay = Some(Overlay::Help),
                'u' => {
                    self.unread_only = !self.unread_only;
                    self.dirty = true;
                }
                'c' => {
                    self.filter_pull_request = None;
                    self.filter_stack = None;
                    self.unread_only = false;
                    self.dirty = true;
                }
                'C' => {
                    self.sort = SortMode::Updated;
                    self.direction = SortDirection::Asc;
                    self.filter_pull_request = None;
                    self.filter_stack = None;
                    self.unread_only = false;
                    self.show_ignored = true;
                    self.dirty = true;
                }
                's' => {
                    self.sort = self.sort.next();
                    self.direction = SortDirection::Asc;
                    self.dirty = true;
                }
                'x' => self.toggle_ignore(),
                'f' => {
                    if let Some(pr) = self.selected_pull_request_owned() {
                        self.filter_pull_request = Some(pr.key);
                        self.filter_stack = None;
                        self.dirty = true;
                    }
                }
                'F' => {
                    if let Some(pr) = self.selected_pull_request_owned() {
                        if let Some(stack) = pr.stack {
                            self.filter_stack = Some(stack.number);
                            self.filter_pull_request = None;
                        } else {
                            self.filter_pull_request = Some(pr.key);
                            self.filter_stack = None;
                        }
                        self.dirty = true;
                    }
                }
                'a' => self.mark_visible_read(),
                'r' => self.mark_selected_read(),
                _ => {}
            },
        }
    }

    fn handle_overlay_key(&mut self, key: Key) {
        let Some(overlay) = self.overlay.clone() else {
            return;
        };
        match overlay {
            Overlay::ActivityDetail {
                activity,
                pull_request,
                scroll,
            } => match key {
                Key::Esc | Key::Quit => self.overlay = None,
                Key::Down => self.scroll_activity(scroll + 1),
                Key::Up => self.scroll_activity(scroll.saturating_sub(1)),
                Key::PageDown => self.scroll_activity(scroll + 10),
                Key::PageUp => self.scroll_activity(scroll.saturating_sub(10)),
                Key::Home => self.scroll_activity(0),
                Key::End => self.scroll_activity(usize::MAX),
                Key::Char('j') => self.scroll_activity(scroll + 1),
                Key::Char('k') => self.scroll_activity(scroll.saturating_sub(1)),
                Key::Enter => {
                    let url = activity.pull_request.url();
                    match self.browser.open(&url) {
                        Ok(()) => self.overlay = None,
                        Err(error) => self.status = Some(error),
                    }
                }
                _ => {
                    let _ = pull_request;
                }
            },
            Overlay::PullRequestDetail {
                pull_request,
                scroll,
            } => match key {
                Key::Esc | Key::Quit => self.overlay = None,
                Key::Down => self.scroll_pull_request(scroll + 1),
                Key::Up => self.scroll_pull_request(scroll.saturating_sub(1)),
                Key::PageDown => self.scroll_pull_request(scroll + 10),
                Key::PageUp => self.scroll_pull_request(scroll.saturating_sub(10)),
                Key::Home => self.scroll_pull_request(0),
                Key::End => self.scroll_pull_request(usize::MAX),
                Key::Char('j') => self.scroll_pull_request(scroll + 1),
                Key::Char('k') => self.scroll_pull_request(scroll.saturating_sub(1)),
                Key::Enter => match self.browser.open(&pull_request.key.url()) {
                    Ok(()) => self.overlay = None,
                    Err(error) => self.status = Some(error),
                },
                _ => {}
            },
            Overlay::Help => match key {
                Key::Esc | Key::Quit | Key::Help | Key::Char('h') | Key::Char('H') => {
                    self.overlay = None
                }
                _ => {}
            },
            Overlay::Settings { selected } => match key {
                Key::Esc | Key::Quit | Key::Char(',') => self.overlay = None,
                Key::Down | Key::Char('j') => {
                    self.overlay = Some(Overlay::Settings {
                        selected: (selected + 1).min(1),
                    })
                }
                Key::Up | Key::Char('k') => {
                    self.overlay = Some(Overlay::Settings {
                        selected: selected.saturating_sub(1),
                    })
                }
                Key::Space | Key::Enter => {
                    if selected == 0 {
                        self.prefs.desktop = Some(!self.prefs.desktop());
                    } else {
                        self.prefs.sound = Some(!self.prefs.sound());
                    }
                    self.dirty = true;
                    self.save_preferences();
                }
                _ => {}
            },
            Overlay::ScopePicker { selected } => self.handle_picker_key(selected, key),
            Overlay::SavePrompt => match key {
                Key::Char('y') => {
                    self.save_preferences();
                    self.should_quit = true;
                }
                Key::Char('n') => self.should_quit = true,
                Key::Esc | Key::Quit => self.overlay = None,
                _ => {}
            },
        }
    }

    fn handle_picker_key(&mut self, selected: usize, key: Key) {
        let repositories = self.known_repositories();
        let focus = FocusRelation::ALL;
        let total = repositories.len() + focus.len();
        match key {
            Key::Esc | Key::Quit => self.overlay = None,
            Key::Down | Key::Char('j') => {
                self.overlay = Some(Overlay::ScopePicker {
                    selected: (selected + 1).min(total.saturating_sub(1)),
                })
            }
            Key::Up | Key::Char('k') => {
                self.overlay = Some(Overlay::ScopePicker {
                    selected: selected.saturating_sub(1),
                })
            }
            Key::Char('m') => {
                let mode = match self.config.watch.effective_mode() {
                    WatchMode::All => WatchMode::Selected,
                    WatchMode::Selected => WatchMode::All,
                };
                match ScopeEditor::set_mode(&self.config_path, mode) {
                    Ok(()) => {
                        self.reload_config();
                        self.reload_view().ok();
                        self.status = None;
                    }
                    Err(error) => {
                        self.status = Some(format!("cannot write configuration: {error}"))
                    }
                }
            }
            Key::Space => {
                if selected < repositories.len() {
                    let repository = repositories[selected].clone();
                    let selected_now = self
                        .config
                        .watch
                        .repositories
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(&repository));
                    let result = if selected_now {
                        ScopeEditor::deselect_repository(&self.config_path, &repository)
                    } else {
                        ScopeEditor::select_repository(&self.config_path, &repository)
                    };
                    match result {
                        Ok(()) => {
                            self.reload_config();
                            self.reload_view().ok();
                            self.status = None;
                        }
                        Err(error) => {
                            self.status = Some(format!("cannot write configuration: {error}"))
                        }
                    }
                } else if let Some(relation) = focus.get(selected - repositories.len()) {
                    let mut relations = self.config.focus.relations.clone();
                    if relations.contains(relation) {
                        relations.retain(|candidate| candidate != relation);
                    } else {
                        relations.push(*relation);
                        relations.sort();
                    }
                    match ScopeEditor::set_focus(&self.config_path, &relations) {
                        Ok(()) => {
                            self.reload_config();
                            self.reload_view().ok();
                            self.status = None;
                        }
                        Err(error) => {
                            self.status = Some(format!("cannot write configuration: {error}"))
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn reload_config(&mut self) {
        if let Ok(config) = AppConfig::load(&self.config_path) {
            self.config = config;
        }
    }

    pub fn known_repositories(&self) -> Vec<String> {
        let mut set: BTreeSet<String> = self.view.known_repositories.iter().cloned().collect();
        for repository in &self.config.watch.repositories {
            set.insert(repository.clone());
        }
        set.into_iter().collect()
    }

    fn scroll_activity(&mut self, scroll: usize) {
        if let Some(Overlay::ActivityDetail {
            activity,
            pull_request,
            ..
        }) = self.overlay.clone()
        {
            let max = self.activity_detail_max_scroll();
            self.overlay = Some(Overlay::ActivityDetail {
                activity,
                pull_request,
                scroll: scroll.min(max),
            });
        }
    }

    fn scroll_pull_request(&mut self, scroll: usize) {
        if let Some(Overlay::PullRequestDetail { pull_request, .. }) = self.overlay.clone() {
            let max = self.pull_request_detail_max_scroll();
            self.overlay = Some(Overlay::PullRequestDetail {
                pull_request,
                scroll: scroll.min(max),
            });
        }
    }

    fn activity_detail_max_scroll(&self) -> usize {
        match &self.overlay {
            Some(Overlay::ActivityDetail {
                activity,
                pull_request,
                ..
            }) => crate::tui::render::activity_detail_lines(activity, pull_request.as_ref())
                .len()
                .saturating_sub(self.detail_viewport.get().max(1)),
            _ => 0,
        }
    }

    fn pull_request_detail_max_scroll(&self) -> usize {
        match &self.overlay {
            Some(Overlay::PullRequestDetail { pull_request, .. }) => {
                crate::tui::render::pull_request_detail_lines(pull_request)
                    .len()
                    .saturating_sub(self.detail_viewport.get().max(1))
            }
            _ => 0,
        }
    }

    fn move_selection(&mut self, delta: isize) {
        match self.focus {
            Pane::Watched => {
                let len = self.sorted_pull_requests().len();
                self.watched_selection = clamp_index(self.watched_selection, len, delta);
            }
            Pane::Activity => {
                let len = self.visible_activity().len();
                self.activity_selection = clamp_index(self.activity_selection, len, delta);
            }
        }
    }

    fn jump_selection(&mut self, start: bool) {
        match self.focus {
            Pane::Watched => {
                self.watched_selection = if start {
                    0
                } else {
                    self.sorted_pull_requests().len().saturating_sub(1)
                }
            }
            Pane::Activity => {
                self.activity_selection = if start {
                    0
                } else {
                    self.visible_activity().len().saturating_sub(1)
                }
            }
        }
    }

    fn clamp_selections(&mut self) {
        let watched = self.sorted_pull_requests().len();
        self.watched_selection = self.watched_selection.min(watched.saturating_sub(1));
        let activity = self.visible_activity().len();
        self.activity_selection = self.activity_selection.min(activity.saturating_sub(1));
    }

    fn open_selected(&mut self) {
        match self.focus {
            Pane::Watched => {
                if let Some(pull_request) = self.selected_pull_request_owned() {
                    self.overlay = Some(Overlay::PullRequestDetail {
                        pull_request,
                        scroll: 0,
                    });
                }
            }
            Pane::Activity => {
                let selected = self
                    .visible_activity()
                    .get(self.activity_selection)
                    .cloned();
                if let Some(activity) = selected {
                    let pull_request = self
                        .view
                        .watched
                        .iter()
                        .find(|pr| pr.key == activity.pull_request)
                        .cloned();
                    self.overlay = Some(Overlay::ActivityDetail {
                        activity,
                        pull_request,
                        scroll: 0,
                    });
                }
            }
        }
    }

    fn mark_selected_read(&mut self) {
        let Some(activity) = self
            .visible_activity()
            .get(self.activity_selection)
            .cloned()
        else {
            return;
        };
        let service = InboxService::new(self.config.clone(), self.store.clone());
        match service.mark_read(&activity.id) {
            Ok(_) => {
                if let Err(error) = self.reload_view() {
                    self.status = Some(error);
                }
            }
            Err(error) => self.status = Some(format!("state error: {error}")),
        }
    }

    fn mark_visible_read(&mut self) {
        let ids: Vec<String> = self
            .visible_activity()
            .iter()
            .map(|activity| activity.id.clone())
            .collect();
        let service = InboxService::new(self.config.clone(), self.store.clone());
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        match service.mark_many_read(refs) {
            Ok(_) => {
                if let Err(error) = self.reload_view() {
                    self.status = Some(error);
                }
            }
            Err(error) => self.status = Some(format!("state error: {error}")),
        }
    }

    fn toggle_ignore(&mut self) {
        let Some(pull_request) = self.selected_pull_request_owned() else {
            return;
        };
        let key = pull_request.key.to_string();
        let mut ignored = self.config.watch.ignored_pull_requests.clone();
        if ignored
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&key))
        {
            ignored.retain(|candidate| !candidate.eq_ignore_ascii_case(&key));
        } else {
            ignored.push(key);
        }
        match ScopeEditor::set_ignored(&self.config_path, &ignored) {
            Ok(()) => {
                self.reload_config();
                if let Err(error) = self.reload_view() {
                    self.status = Some(error);
                }
            }
            Err(error) => self.status = Some(format!("cannot write configuration: {error}")),
        }
    }

    pub fn save_preferences(&self) {
        self.preferences().write(&self.ui_path);
    }

    pub fn preferences(&self) -> UiPrefs {
        let mut prefs = self.prefs.clone();
        prefs.sort = Some(self.sort);
        prefs.direction = Some(self.direction);
        prefs.filter_pull_request = self.filter_pull_request.as_ref().map(ToString::to_string);
        prefs.filter_stack = self.filter_stack;
        prefs.unread_only = Some(self.unread_only);
        prefs.show_ignored = Some(self.show_ignored);
        prefs
    }
}

fn compare_units(
    left: Option<&PullRequestView>,
    right: Option<&PullRequestView>,
    mode: SortMode,
    direction: SortDirection,
) -> std::cmp::Ordering {
    let (Some(left), Some(right)) = (left, right) else {
        return std::cmp::Ordering::Equal;
    };
    let (left_primary, right_primary) = match mode {
        SortMode::Updated | SortMode::Age => (
            timestamp_key(left.created_at),
            timestamp_key(right.created_at),
        ),
        SortMode::Name => (key_sort_string(&left.key), key_sort_string(&right.key)),
        SortMode::State => (
            left.state.as_str().to_string(),
            right.state.as_str().to_string(),
        ),
    };
    let mut ordering = left_primary.cmp(&right_primary);
    if direction == SortDirection::Desc {
        ordering = ordering.reverse();
    }
    if ordering == std::cmp::Ordering::Equal {
        key_sort_string(&left.key).cmp(&key_sort_string(&right.key))
    } else {
        ordering
    }
}

fn key_sort_string(key: &PullRequestKey) -> String {
    format!("{}#{:020}", key.repository(), key.number())
}

fn timestamp_key(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|value| format!("{:020}", value.timestamp()))
        .unwrap_or_default()
}

fn clamp_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    if delta >= 0 {
        (current + delta as usize).min(len - 1)
    } else {
        current.saturating_sub(delta.unsigned_abs())
    }
}

impl UiPrefs {
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn write(&self, path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }
}

#[allow(dead_code)]
fn _time(_: DateTime<Utc>) {}
