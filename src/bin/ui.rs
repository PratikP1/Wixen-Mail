//! Wixen Mail UI launcher
//!
//! This binary launches the graphical user interface for Wixen Mail.

use wixen_mail::{
    common::logging::{init_logging, LoggerConfig},
    presentation::UI,
};

fn main() {
    // Initialize logging
    let _log_guard = init_logging(LoggerConfig::default())
        .map_err(|e| eprintln!("Failed to initialize logging: {e}"))
        .ok();

    tracing::info!("Starting Wixen Mail UI");

    // Create and run UI
    let ui = UI::new().expect("Failed to create UI");
    if let Err(e) = ui.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
