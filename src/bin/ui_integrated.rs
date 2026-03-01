//! Wixen Mail UI launcher with full IMAP/SMTP integration
//!
//! This binary launches the wxdragon-based graphical user interface for Wixen Mail
//! with real IMAP/SMTP connectivity.

use wixen_mail::{
    common::logging::{init_logging, LoggerConfig},
    presentation::WxMailApp,
};

fn main() {
    // Initialize logging
    let _log_guard = init_logging(LoggerConfig::default())
        .map_err(|e| eprintln!("Failed to initialize logging: {e}"))
        .ok();

    tracing::info!("Starting Wixen Mail with wxdragon UI");

    // Create and run wxdragon app
    let app = WxMailApp::new().expect("Failed to create wxdragon app");
    if let Err(e) = app.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
