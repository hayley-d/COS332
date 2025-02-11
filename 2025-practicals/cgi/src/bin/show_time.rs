use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    timezone: i32,
    country: String,
    capital: String,
}

fn write_config(config: &Config) {
    let yaml = serde_yaml::to_string(config).expect("Failed to serialize YAML");
    fs::write("backend.yaml", yaml).expect("Failed to write file");
}
