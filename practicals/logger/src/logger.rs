use std::fmt::Display;

pub struct Logger {
    level: Level,
}

impl Logger {
    pub fn new(config: config::Config) -> Logger {
        Logger
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
