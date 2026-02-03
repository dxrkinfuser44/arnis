//! Structured logging infrastructure for Arnis
//!
//! Provides leveled logging with consistent formatting across the application.
//! Supports console output with colors and optional file logging.

use colored::Colorize;
use std::fmt;
use std::sync::Mutex;
use std::time::SystemTime;

/// Log level priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    /// Get the color for this log level
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Global minimum log level - messages below this level are filtered out
static MIN_LOG_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Info);

/// Whether to show timestamps in log output
static SHOW_TIMESTAMPS: Mutex<bool> = Mutex::new(true);

/// Whether to use colored output
static USE_COLORS: Mutex<bool> = Mutex::new(true);

/// Initialize the logging system with the specified minimum level
pub fn init(level: LogLevel, show_timestamps: bool, use_colors: bool) {
    if let Ok(mut min_level) = MIN_LOG_LEVEL.lock() {
        *min_level = level;
    }
    if let Ok(mut timestamps) = SHOW_TIMESTAMPS.lock() {
        *timestamps = show_timestamps;
    }
    if let Ok(mut colors) = USE_COLORS.lock() {
        *colors = use_colors;
    }
}

/// Get the current minimum log level
fn get_min_level() -> LogLevel {
    MIN_LOG_LEVEL.lock().map(|l| *l).unwrap_or(LogLevel::Info)
}

/// Check if timestamps should be shown
fn show_timestamps() -> bool {
    SHOW_TIMESTAMPS.lock().map(|t| *t).unwrap_or(true)
}

/// Check if colors should be used
fn use_colors() -> bool {
    USE_COLORS.lock().map(|c| *c).unwrap_or(true)
}

/// Format the timestamp for log output
fn format_timestamp() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60,
            millis
        )
    } else {
        String::from("??:??:??.???")
    }
}

/// Core logging function
fn log(level: LogLevel, module: &str, message: &str) {
    // Check if this message should be logged
    if level > get_min_level() {
        return;
    }

    let timestamp = if show_timestamps() {
        format!("[{}] ", format_timestamp())
    } else {
        String::new()
    };

    let formatted = if use_colors() {
        let level_str = match level {
            LogLevel::Error => level.as_str().red().bold(),
            LogLevel::Warning => level.as_str().yellow(),
            LogLevel::Info => level.as_str().white(),
            LogLevel::Debug => level.as_str().blue(),
            LogLevel::Trace => level.as_str().magenta(),
        };
        format!("{}{} [{}] {}", timestamp, level_str, module, message)
    } else {
        format!("{}{} [{}] {}", timestamp, level.as_str(), module, message)
    };

    // Output to appropriate stream
    match level {
        LogLevel::Error => eprintln!("{}", formatted),
        _ => println!("{}", formatted),
    }
}

/// Log an error message
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::log_error(&format!($($arg)*))
    };
}

/// Log a warning message
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::log_warn(&format!($($arg)*))
    };
}

/// Log an info message
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::log_info(&format!($($arg)*))
    };
}

/// Log a debug message
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::log_debug(&format!($($arg)*))
    };
}

/// Log a trace message
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::logger::log_trace(&format!($($arg)*))
    };
}

/// Public logging functions
pub fn log_error(message: &str) {
    log(LogLevel::Error, module_path!(), message);
}

pub fn log_warn(message: &str) {
    log(LogLevel::Warning, module_path!(), message);
}

pub fn log_info(message: &str) {
    log(LogLevel::Info, module_path!(), message);
}

pub fn log_debug(message: &str) {
    log(LogLevel::Debug, module_path!(), message);
}

#[allow(dead_code)]
pub fn log_trace(message: &str) {
    log(LogLevel::Trace, module_path!(), message);
}

/// Log with explicit module name (useful when module_path!() doesn't give desired result)
#[allow(dead_code)]
pub fn log_with_module(level: LogLevel, module: &str, message: &str) {
    let _ = level; // Suppress unused warning when logging is disabled
    let _ = module;
    let _ = message;
    // Re-implement the core logic inline to avoid borrowing issues
    let min_level = get_min_level();
    if level > min_level {
        return;
    }

    let timestamp = if show_timestamps() {
        format!("[{}] ", format_timestamp())
    } else {
        String::new()
    };

    let formatted = if use_colors() {
        let level_str = match level {
            LogLevel::Error => level.as_str().red().bold(),
            LogLevel::Warning => level.as_str().yellow(),
            LogLevel::Info => level.as_str().white(),
            LogLevel::Debug => level.as_str().blue(),
            LogLevel::Trace => level.as_str().magenta(),
        };
        format!("{}{} [{}] {}", timestamp, level_str, module, message)
    } else {
        format!("{}{} [{}] {}", timestamp, level.as_str(), module, message)
    };

    match level {
        LogLevel::Error => eprintln!("{}", formatted),
        _ => println!("{}", formatted),
    }
}

/// Set minimum log level at runtime
#[allow(dead_code)]
pub fn set_log_level(level: LogLevel) {
    if let Ok(mut min_level) = MIN_LOG_LEVEL.lock() {
        *min_level = level;
    }
}

/// Get the current log level
#[allow(dead_code)]
pub fn current_log_level() -> LogLevel {
    get_min_level()
}

#[allow(dead_code)]
pub fn should_log(level: LogLevel) -> bool {
    level <= get_min_level()
}

/// Progress logging helper - for structured progress updates
#[allow(dead_code)]
pub struct ProgressLogger {
    total_steps: u32,
    current_step: u32,
    operation_name: String,
}

#[allow(dead_code)]
impl ProgressLogger {
    /// Create a new progress logger
    pub fn new(operation_name: &str, total_steps: u32) -> Self {
        let logger = Self {
            total_steps,
            current_step: 0,
            operation_name: operation_name.to_string(),
        };
        info!("Starting: {} ({} steps)", operation_name, total_steps);
        logger
    }

    /// Update progress and log if significant change
    #[allow(clippy::manual_is_multiple_of)]
    pub fn update(&mut self, step: u32) {
        self.current_step = step;
        let percentage = (self.current_step as f32 / self.total_steps as f32) * 100.0;

        // Log at 25%, 50%, 75%, and 100%
        if self.current_step % (self.total_steps / 4).max(1) == 0
            || self.current_step == self.total_steps
        {
            info!(
                "{}: {}/{} ({:.1}%)",
                self.operation_name, self.current_step, self.total_steps, percentage
            );
        }
    }

    /// Complete the operation
    pub fn complete(&self) {
        info!("Completed: {}", self.operation_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
    }

    #[test]
    fn test_progress_logger() {
        let mut logger = ProgressLogger::new("Test Operation", 100);
        logger.update(25);
        logger.update(50);
        logger.update(100);
        logger.complete();
    }
}
