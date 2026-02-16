//! Wixen Mail UI launcher with full IMAP/SMTP integration
//!
//! This binary launches the fully integrated graphical user interface for Wixen Mail
//! with real IMAP/SMTP connectivity.

use wixen_mail::{
    common::logging::{init_logging, LoggerConfig},
    presentation::IntegratedUI,
};

fn main() {
    // Initialize logging
    let _log_guard = init_logging(LoggerConfig::default())
        .map_err(|e| eprintln!("Failed to initialize logging: {e}"))
        .ok();

    tracing::info!("Starting Wixen Mail Integrated UI");

    // Create and run integrated UI
    let ui = IntegratedUI::new().expect("Failed to create integrated UI");
    if let Err(e) = ui.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
