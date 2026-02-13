//! Logging framework with privacy-aware features
//!
//! Provides structured logging with file rotation and privacy protection.

use std::path::PathBuf;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

/// Privacy-aware string wrapper that masks sensitive data in logs
#[derive(Debug)]
pub struct SensitiveString(String);

impl SensitiveString {
    /// Create a new sensitive string
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Get the actual value (use sparingly)
    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "***REDACTED***")
    }
}

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
    pub fn to_tracing_level(&self) -> Level {
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
            log_dir: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("wixen-mail")
                .join("logs"),
            log_file_prefix: "wixen-mail".to_string(),
            console_logging: true,
        }
    }
}

/// Initialize the logging system
///
/// Returns a WorkerGuard that must be kept alive for the duration of the program
pub fn init_logging(config: LoggerConfig) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    if config.log_to_file {
        std::fs::create_dir_all(&config.log_dir)?;
    }

    // Set up file appender with rotation
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, &config.log_file_prefix);
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
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() > 2 {
            format!("{}***{}", &local[..2], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        "***@***".to_string()
    }
}

/// Mask password for privacy
pub fn mask_password(_password: &str) -> &'static str {
    "***REDACTED***"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_string() {
        let sensitive = SensitiveString::new("secret-password".to_string());
        assert_eq!(sensitive.reveal(), "secret-password");
        assert_eq!(format!("{}", sensitive), "***REDACTED***");
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
    fn test_mask_password() {
        assert_eq!(mask_password("secret123"), "***REDACTED***");
        assert_eq!(mask_password(""), "***REDACTED***");
    }
}
