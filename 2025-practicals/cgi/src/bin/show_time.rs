use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    timezone: i32,
    country: String,
    city: String,
}

fn read_config() -> Config {
    let content = fs::read_to_string("backend.yaml").expect("Failed to read backend.yaml");
    serde_yaml::from_str(&content).expect("Failed to parse YAML")
}

fn get_time(offset: i32) -> String {
    let utc_now = Utc::now();
    let tz_offset = FixedOffset::east_opt(offset * 3600);
    utc_now
        .with_timezone(&tz_offset)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = read_config();
    let current_time = get_time(config.timezone);
    println!("<html><head><title>Current time</title></head><body><h1>The Current Time in {}, {} is {}</h1><a href='set_south_africa_time.cgi'>Switch South African Time</a></br><a href='set_edinburgh_time.cgi'>Switch United Kingdom Time</a></body></html>",config.city,config.country,current_time);
    Ok(())
}
