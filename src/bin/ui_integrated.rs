#![windows_subsystem = "windows"]

//! Wixen Mail UI launcher with full IMAP/SMTP integration
//!
//! This binary launches the wxdragon-based graphical user interface for Wixen Mail
//! with real IMAP/SMTP connectivity.

use wixen_mail::{
    common::logging::{LoggerConfig, init_logging},
    presentation::WxMailApp,
};

fn main() {
    // Install panic hook FIRST — captures crashes even without a console.
    install_panic_hook();

    let _log_guard = init_logging(LoggerConfig::default())
        .map_err(|e| log_crash(&format!("Failed to initialize logging: {e}")))
        .ok();

    tracing::info!(
        "Starting Wixen Mail (ui_integrated) v{}",
        env!("CARGO_PKG_VERSION")
    );

    match WxMailApp::new() {
        Ok(app) => {
            if let Err(e) = app.run() {
                tracing::error!("UI error: {}", e);
                log_crash(&format!("UI run error: {}", e));
                show_error_dialog(&format!("Wixen Mail failed to run:\n{}", e));
                std::process::exit(1);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create wxdragon app: {}", e);
            log_crash(&format!("Init error: {}", e));
            show_error_dialog(&format!("Wixen Mail failed to start:\n{}", e));
            std::process::exit(1);
        }
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = format!(
            "PANIC at {}\n  {}\n  Wixen Mail v{}\n  Time: {:?}",
            location,
            payload,
            env!("CARGO_PKG_VERSION"),
            std::time::SystemTime::now(),
        );

        tracing::error!("{}", message);
        log_crash(&message);
    }));
}

fn log_crash(message: &str) {
    let crash_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("wixen-mail")
        .join("logs");
    let _ = std::fs::create_dir_all(&crash_dir);
    let crash_file = crash_dir.join("crash.log");
    let timestamped = format!(
        "[{}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        message
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&crash_file)
        .and_then(|mut f| std::io::Write::write_all(&mut f, timestamped.as_bytes()));
}

fn show_error_dialog(message: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;

        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, flags: u32)
            -> i32;
        }

        fn to_wide(s: &str) -> Vec<u16> {
            OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        let text = to_wide(message);
        let caption = to_wide("Wixen Mail Error");
        unsafe {
            MessageBoxW(ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("{}", message);
    }
}
