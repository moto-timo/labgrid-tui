use std::collections::BTreeMap;

use crate::grpc::client::CoordinatorEvent;
use crate::model::{Exporter, PlaceInfo, ResourceInfo, ResourcePath};

/// Which main view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Places,
    Resources,
    Exporters,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Places => "Places",
            View::Resources => "Resources",
            View::Exporters => "Exporters",
        }
    }

    pub fn all() -> &'static [View] {
        &[View::Places, View::Resources, View::Exporters]
    }

    pub fn index(&self) -> usize {
        match self {
            View::Places => 0,
            View::Resources => 1,
            View::Exporters => 2,
        }
    }
}

/// Input mode for the command/filter bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
    Command,
}

/// Central application state — the single source of truth.
pub struct App {
    // Connection
    pub connected: bool,
    pub coordinator_url: String,
    pub status_message: String,

    // Data
    pub places: BTreeMap<String, PlaceInfo>,
    pub resources: BTreeMap<ResourcePath, ResourceInfo>,

    // View state
    pub view: View,
    pub selected_index: usize,
    pub detail_open: bool,
    pub help_open: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub filter: Option<String>,

    // Scroll state for detail pane
    pub detail_scroll: u16,

    // Sync tracking
    pub last_sync_id: u64,
    pub synced: bool,

    // Quit signal
    pub should_quit: bool,
}

impl App {
    pub fn new(coordinator_url: String) -> Self {
        Self {
            connected: false,
            coordinator_url,
            status_message: "connecting...".into(),

            places: BTreeMap::new(),
            resources: BTreeMap::new(),

            view: View::Places,
            selected_index: 0,
            detail_open: false,
            help_open: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            filter: None,

            detail_scroll: 0,
            last_sync_id: 0,
            synced: false,
            should_quit: false,
        }
    }

    // --- Data access ---

    /// Get filtered place names in sorted order.
    pub fn filtered_place_names(&self) -> Vec<String> {
        self.places
            .keys()
            .filter(|name| self.matches_filter(name))
            .cloned()
            .collect()
    }

    /// Get filtered resource paths in sorted order.
    pub fn filtered_resource_paths(&self) -> Vec<ResourcePath> {
        self.resources
            .keys()
            .filter(|path| self.matches_filter(&path.to_string()))
            .cloned()
            .collect()
    }

    /// Build exporter summaries from current resources.
    pub fn exporters(&self) -> Vec<Exporter> {
        let mut by_exporter: BTreeMap<String, Vec<&ResourceInfo>> = BTreeMap::new();
        for res in self.resources.values() {
            by_exporter
                .entry(res.path.exporter.clone())
                .or_default()
                .push(res);
        }

        by_exporter
            .iter()
            .filter(|(name, _)| self.matches_filter(name))
            .map(|(name, resources)| Exporter::from_resources(name, resources))
            .collect()
    }

    /// Number of items in the current view (after filtering).
    pub fn item_count(&self) -> usize {
        match self.view {
            View::Places => self.filtered_place_names().len(),
            View::Resources => self.filtered_resource_paths().len(),
            View::Exporters => self.exporters().len(),
        }
    }

    fn matches_filter(&self, text: &str) -> bool {
        match &self.filter {
            Some(f) if !f.is_empty() => {
                text.to_lowercase().contains(&f.to_lowercase())
            }
            _ => true,
        }
    }

    // --- State mutations (Elm-style update) ---

    pub fn handle_coordinator_event(&mut self, event: CoordinatorEvent) {
        match event {
            CoordinatorEvent::Connected => {
                self.connected = true;
                self.status_message = "connected".into();
            }
            CoordinatorEvent::Disconnected(reason) => {
                self.connected = false;
                self.status_message = format!("disconnected: {reason}");
            }
            CoordinatorEvent::ResourceUpdate(info) => {
                self.resources.insert(info.path.clone(), info);
            }
            CoordinatorEvent::ResourceRemoved(path) => {
                self.resources.remove(&path);
            }
            CoordinatorEvent::PlaceUpdate(info) => {
                self.places.insert(info.name.clone(), info);
            }
            CoordinatorEvent::PlaceRemoved(name) => {
                self.places.remove(&name);
            }
            CoordinatorEvent::SyncComplete(id) => {
                self.last_sync_id = id;
                self.synced = true;
                self.status_message = format!(
                    "synced — {} places, {} resources",
                    self.places.len(),
                    self.resources.len()
                );
            }
        }
    }

    pub fn switch_view(&mut self, view: View) {
        self.view = view;
        self.selected_index = 0;
        self.detail_open = false;
        self.detail_scroll = 0;
    }

    pub fn select_next(&mut self) {
        let count = self.item_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1).min(count - 1);
        }
        self.detail_scroll = 0;
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
        self.detail_scroll = 0;
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
        self.detail_scroll = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.item_count();
        if count > 0 {
            self.selected_index = count - 1;
        }
        self.detail_scroll = 0;
    }

    pub fn toggle_detail(&mut self) {
        self.detail_open = !self.detail_open;
        self.detail_scroll = 0;
    }

    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
    }

    pub fn enter_filter_mode(&mut self) {
        self.input_mode = InputMode::Filter;
        self.input_buffer.clear();
    }

    pub fn enter_command_mode(&mut self) {
        self.input_mode = InputMode::Command;
        self.input_buffer.clear();
    }

    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn submit_input(&mut self) {
        match self.input_mode {
            InputMode::Filter => {
                if self.input_buffer.is_empty() {
                    self.filter = None;
                } else {
                    self.filter = Some(self.input_buffer.clone());
                }
                self.selected_index = 0;
            }
            InputMode::Command => {
                self.execute_command(&self.input_buffer.clone());
            }
            InputMode::Normal => {}
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.selected_index = 0;
    }

    fn execute_command(&mut self, cmd: &str) {
        let cmd = cmd.trim().to_lowercase();
        match cmd.as_str() {
            "q" | "quit" => self.should_quit = true,
            "places" | "p" => self.switch_view(View::Places),
            "resources" | "r" => self.switch_view(View::Resources),
            "exporters" | "e" => self.switch_view(View::Exporters),
            _ => {
                self.status_message = format!("unknown command: {cmd}");
            }
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }
}
