//! Logging framework with privacy-aware features
//!
//! Provides structured logging with file rotation and privacy protection.

use std::path::PathBuf;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt};

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Convert to tracing Level
    fn to_tracing_level(self) -> Level {
        match self {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

/// Logger configuration
pub struct LoggerConfig {
    /// Log level
    pub level: LogLevel,
    /// Log to file
    pub log_to_file: bool,
    /// Log directory
    pub log_dir: PathBuf,
    /// Log file prefix
    pub log_file_prefix: String,
    /// Enable console logging
    pub console_logging: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_to_file: true,
            log_dir: default_log_dir(),
            log_file_prefix: "wixen-mail".to_string(),
            console_logging: true,
        }
    }
}

/// Where logs go when nobody says otherwise.
///
/// Falling back to the current directory, which this used to do, scattered logs
/// into whichever folder the shortcut started in and made them impossible to
/// ask somebody for. The temporary folder is at least a fixed place to look.
pub fn default_log_dir() -> PathBuf {
    crate::common::paths::AppPaths::resolve()
        .map(|paths| paths.logs_dir())
        .unwrap_or_else(|_| std::env::temp_dir().join("wixen-mail").join("logs"))
}

/// Initialize the logging system
///
/// Returns a WorkerGuard that must be kept alive for the duration of the program
pub fn init_logging(config: LoggerConfig) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    if config.log_to_file {
        std::fs::create_dir_all(&config.log_dir)?;
    }

    // Set up file appender with rotation, files named wixen-mail.YYYY-MM-DD.log
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(&config.log_file_prefix)
        .filename_suffix("log")
        .build(&config.log_dir)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Create filter
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "wixen_mail={}",
            config.level.to_tracing_level().as_str()
        ))
    });

    // Create layers
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(filter.clone());

    let subscriber = Registry::default().with(file_layer);

    // Add console layer if enabled
    if config.console_logging {
        let console_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_filter(filter);
        let subscriber = subscriber.with(console_layer);
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        tracing::subscriber::set_global_default(subscriber)?;
    }

    tracing::info!("Logging initialized at level: {:?}", config.level);

    Ok(guard)
}

/// Mask email address for privacy
pub fn mask_email(email: &str) -> String {
    let Some(at_pos) = email.find('@') else {
        return "***@***".to_string();
    };
    let (local, domain) = email.split_at(at_pos);

    // Counted in characters, not bytes. RFC 6531 allows a non-ASCII local
    // part, and "aé" is three bytes, so taking two bytes would cut the é in
    // half and panic. This runs on every logged send.
    let mut chars = local.chars();
    let prefix: String = chars.by_ref().take(2).collect();
    if chars.next().is_some() {
        format!("{}***{}", prefix, domain)
    } else {
        format!("***{}", domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_level_can_be_asked_for_by_name() {
        // Found by mutation testing: only some of the arms were exercised, so
        // deleting the one for "warn" or "trace" changed nothing any test
        // could see. These come off the command line and out of the settings
        // file, and a level that silently falls back is a person who asked for
        // more detail in a bug report and did not get it.
        for (written, expected) in [
            ("error", LogLevel::Error),
            ("warn", LogLevel::Warn),
            ("info", LogLevel::Info),
            ("debug", LogLevel::Debug),
            ("trace", LogLevel::Trace),
        ] {
            assert_eq!(
                LogLevel::parse(written),
                Some(expected),
                "{written} was not understood"
            );
            assert_eq!(
                LogLevel::parse(&written.to_uppercase()),
                Some(expected),
                "{written} was not understood in capitals"
            );
        }
        assert_eq!(LogLevel::parse("chatty"), None);
    }

    #[test]
    fn test_the_log_folder_is_somewhere_real() {
        // Not the empty path, which is what this became under mutation and
        // which would put the log wherever the shortcut happened to start.
        // Logs nobody can find are logs nobody can send with a bug report.
        let dir = default_log_dir();

        assert!(dir.is_absolute(), "the log folder is not an absolute path");
        assert!(dir.ends_with("logs"), "{dir:?} is not a logs folder");
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::Error.to_tracing_level(), tracing::Level::ERROR);
        assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
        assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::parse("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::parse("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("invalid"), None);
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("user@example.com"), "us***@example.com");
        assert_eq!(mask_email("a@example.com"), "***@example.com");
        assert_eq!(mask_email("test"), "***@***");
    }

    #[test]
    fn test_mask_email_handles_internationalised_addresses() {
        // RFC 6531 allows non-ASCII local parts, and "aé" is three bytes, so
        // taking the first two bytes lands inside the é. Masking runs on every
        // logged send, so this crashed on a legitimate address.
        assert_eq!(mask_email("a\u{e9}b@example.com"), "a\u{e9}***@example.com");
        assert_eq!(
            mask_email("\u{4f60}\u{597d}@example.com"),
            "***@example.com"
        );
        assert_eq!(
            mask_email("\u{4f60}\u{597d}\u{4e16}@example.com"),
            "\u{4f60}\u{597d}***@example.com"
        );
    }

    #[test]
    fn test_mask_email_never_panics_on_odd_input() {
        for address in [
            "",
            "@",
            "@example.com",
            "user@",
            "\u{e9}@\u{e9}",
            "\u{feff}@x",
            "a\u{0}b@example.com",
            "@@@",
        ] {
            let _ = mask_email(address);
        }
    }
}
