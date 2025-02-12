use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    timezone: i32,
    country: String,
    city: String,
}

fn write_config(config: &Config) {
    let yaml = serde_yaml::to_string(config).expect("Failed to serialize YAML");
    fs::write("backend.yaml", yaml).expect("Failed to write file");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let new_config: Config = Config {
        timezone: 2,
        country: "South Africa".to_string(),
        city: "Pretoria".to_string(),
    };

    write_config(&new_config);

    println!("<html><head><meta http-equiv='refresh' content='0;url=show_time.cgi'></head><body><h3>Switching to South Africa time</h3></body></html>");
    Ok(())
}
