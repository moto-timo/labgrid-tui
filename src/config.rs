use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

/// A k9s-style terminal UI for labgrid infrastructure monitoring.
#[derive(Parser, Debug)]
#[command(name = "labgrid-tui", version, about)]
pub struct Cli {
    /// Coordinator URL (e.g., ws://coordinator:20408/ws or just host:port)
    #[arg(short, long, env = "LG_COORDINATOR")]
    pub coordinator: Option<String>,

    /// Path to config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// UI tick rate in milliseconds
    #[arg(long, default_value = "250")]
    pub tick_rate: u64,

    /// Log file path (logs to file instead of stderr)
    #[arg(long)]
    pub log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub coordinator: CoordinatorConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct CoordinatorConfig {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: default_tick_rate(),
        }
    }
}

fn default_tick_rate() -> u64 {
    250
}

impl Config {
    /// Load config from a TOML file.
    pub fn load(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Find the default config file location.
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "labgrid-tui")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }
}

/// Resolve the final coordinator URL from CLI args, config file, and environment.
///
/// Normalizes the coordinator URL:
/// - `http://host:port` or `https://...` → used as-is (plain HTTP/2 gRPC)
/// - `ws://host:port/ws` or `wss://...` → used as-is (WebSocket transport)
/// - `host:port` (no scheme) → converted to `http://host:port`
pub fn resolve_coordinator_url(cli: &Cli, config: &Config) -> Option<String> {
    // CLI flag takes priority, then config file, then env (handled by clap)
    let raw = cli
        .coordinator
        .clone()
        .or_else(|| config.coordinator.url.clone())?;

    Some(normalize_coordinator_url(&raw))
}

fn normalize_coordinator_url(raw: &str) -> String {
    if raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("ws://")
        || raw.starts_with("wss://")
    {
        raw.to_string()
    } else {
        // Bare host:port → default to plain HTTP/2 gRPC
        format!("http://{}", raw)
    }
}
