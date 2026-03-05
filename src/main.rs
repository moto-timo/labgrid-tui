mod app;
mod config;
mod event;
mod grpc;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tracing::info;

use app::App;
use config::{Cli, Config};
use event::{handle_terminal_event, spawn_terminal_event_reader, AppEvent};
use grpc::client::{CoordinatorClient, CoordinatorEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    if let Some(ref log_file) = cli.log_file {
        let file = std::fs::File::create(log_file)
            .with_context(|| format!("failed to create log file: {}", log_file.display()))?;
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("labgrid_tui=debug".parse().unwrap()),
            )
            .init();
    } else {
        // When running a TUI, we can't log to stderr — skip init unless log file given.
    }

    // Load config
    let config = if let Some(ref path) = cli.config {
        Config::load(path)?
    } else if let Some(default_path) = Config::default_path() {
        if default_path.exists() {
            Config::load(&default_path).unwrap_or_default()
        } else {
            Config::default()
        }
    } else {
        Config::default()
    };

    // Resolve coordinator URL
    let coordinator_url = config::resolve_coordinator_url(&cli, &config);
    let Some(coordinator_url) = coordinator_url else {
        bail!(
            "no coordinator URL specified.\n\
             Use --coordinator <URL>, set LG_COORDINATOR env var, \
             or add [coordinator] url to config file."
        );
    };

    let tick_rate = Duration::from_millis(cli.tick_rate);

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal, coordinator_url, tick_rate).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("Error: {e:#}");
    }

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    coordinator_url: String,
    tick_rate: Duration,
) -> Result<()> {
    let mut app = App::new(coordinator_url.clone());

    // Channel for all events
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // Spawn terminal event reader
    spawn_terminal_event_reader(event_tx.clone(), tick_rate);

    // Spawn gRPC client
    let grpc_event_tx = event_tx.clone();
    let grpc_client = CoordinatorClient::new(coordinator_url);

    tokio::spawn(async move {
        loop {
            if let Err(e) = grpc_client.run(grpc_event_tx.clone()).await {
                let _ = grpc_event_tx.send(AppEvent::Coordinator(
                    CoordinatorEvent::Disconnected(e.to_string()),
                ));
            }
            // Wait before reconnecting
            tokio::time::sleep(Duration::from_secs(5)).await;
            info!("attempting to reconnect to coordinator...");
        }
    });

    // Main event loop
    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Wait for event
        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Terminal(ev) => {
                    if handle_terminal_event(&mut app, ev) {
                        break;
                    }
                }
                AppEvent::Tick => {
                    // Periodic refresh — nothing specific needed in read-only mode
                }
                AppEvent::Coordinator(ev) => {
                    app.handle_coordinator_event(ev);
                }
            }
        } else {
            break;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
