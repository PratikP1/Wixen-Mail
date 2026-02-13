//! Wixen Mail UI launcher with full IMAP/SMTP integration
//!
//! This binary launches the fully integrated graphical user interface for Wixen Mail
//! with real IMAP/SMTP connectivity.

use wixen_mail::presentation::IntegratedUI;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Wixen Mail Integrated UI");

    // Create and run integrated UI
    let ui = IntegratedUI::new().expect("Failed to create integrated UI");
    if let Err(e) = ui.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
