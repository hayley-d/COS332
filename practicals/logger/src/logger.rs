use std::fmt::Display;
use std::sync::Arc;

use chrono::Local;
use log::Log;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;

use crate::config::Config;

pub struct Logger {
    config: Config,
    error_file: Arc<Mutex<BufWriter<File>>>,
    log_file: Arc<Mutex<BufWriter<File>>>,
}

impl Logger {
    /// Create a new logger that can be independently used.
    ///
    pub async fn new(config: Config) -> Logger {
        let error_file = Arc::new(Mutex::new(BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config.error_file)
                .await
                .expect("Failed to open error log file"),
        )));

        let log_file = Arc::new(Mutex::new(BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config.log_file)
                .await
                .expect("Failed to open log log file"),
        )));

        return Logger {
            config,
            error_file,
            log_file,
        };
    }

    pub async fn log_error(&self, message: &str, line: u32) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let log_message = format!(
            "{} [ERROR] {}:{} - {}\n",
            timestamp, self.config.error_file, line, message
        );

        let mut error_file = self.error_file.lock().await;
        match error_file.write_all(log_message.as_bytes()).await {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing to error log: {:?}", e),
        };

        match error_file.flush().await {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing to error log: {:?}", e),
        };
    }

    pub async fn log_info(&self, message: &str, line: u32) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let log_message = format!(
            "{} [INFO] {}:{} - {}\n",
            timestamp, self.config.log_file, line, message
        );

        let mut log_file = self.log_file.lock().await;
        match log_file.write_all(log_message.as_bytes()).await {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing to info log: {:?}", e),
        };

        match log_file.flush().await {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing to info log: {:?}", e),
        };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Level {
    Info,
    Error,
    Warn,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Info => write!(f, "[INFO]"),
            Level::Warn => write!(f, "[WARN]"),
            Level::Error => write!(f, "[ERROR]"),
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Level::Info => match other {
                Level::Info => Some(std::cmp::Ordering::Equal),
                Level::Warn => Some(std::cmp::Ordering::Less),
                Level::Error => Some(std::cmp::Ordering::Less),
            },
            Level::Warn => match other {
                Level::Info => Some(std::cmp::Ordering::Greater),
                Level::Warn => Some(std::cmp::Ordering::Equal),
                Level::Error => Some(std::cmp::Ordering::Less),
            },
            Level::Error => match other {
                Level::Info => Some(std::cmp::Ordering::Greater),
                Level::Warn => Some(std::cmp::Ordering::Greater),
                Level::Error => Some(std::cmp::Ordering::Equal),
            },
        }
    }
}

pub trait SharedLogger: Log {
    /// Returns the set level for the logger.
    ///
    /// # Example
    /// ```
    /// use logger::*;
    /// fn main() {
    ///     let logger = Logger::new(Level::Info,Config::default());
    ///     println!("{}",logger.level());
    /// }
    /// ```
    fn level(&self) -> Level;

    /// Inspect the config for a logger.
    ///
    /// # Example
    /// ```
    /// use logger::*;
    /// fn main() {
    ///     let logger = Logger::new(Level::Info,Config::default());
    ///     println!("{:?}",logger.config());
    /// }
    /// ```
    fn config(&self) -> Option<&Config>;

    /// Returns the logger as a Log trait object
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
