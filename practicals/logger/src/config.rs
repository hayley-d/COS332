pub struct Config {
    pub error_file: String,
    pub log_file: String,
}

impl Config {
    pub fn new(error_file: String, log_file: String) -> Self {
        return Config {
            error_file,
            log_file,
        };
    }
}
