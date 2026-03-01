use wixen_mail::common::logging::{init_logging, LoggerConfig};
use wixen_mail::presentation::WxMailApp;

fn main() {
    let _log_guard = init_logging(LoggerConfig::default()).ok();
    tracing::info!("Starting Wixen Mail v{}", env!("CARGO_PKG_VERSION"));

    let app = WxMailApp::new().expect("Failed to initialize Wixen Mail");
    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
