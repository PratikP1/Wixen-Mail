//! Wixen Mail UI launcher
//!
//! This binary launches the graphical user interface for Wixen Mail.

use wixen_mail::presentation::UI;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Wixen Mail UI");

    // Create and run UI
    let ui = UI::new().expect("Failed to create UI");
    if let Err(e) = ui.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
