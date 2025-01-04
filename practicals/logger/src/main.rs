use logger::config::Config;
use logger::logger::Logger;
use logger::{error, info};

fn main() {
    // Create a configuration
    let config = Config::new("error.log".to_string(), "general.log".to_string());

    // Initialize the logger
    let logger = Logger::new(config);

    // Log messages

    info!(logger, "Server started successfully");
    error!(logger, "Failed to bind to port 8080");
}
