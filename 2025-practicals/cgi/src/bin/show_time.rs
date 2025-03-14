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
    let content = fs::read_to_string("static/backend.yaml").expect("Failed to read backend.yaml");
    serde_yaml::from_str(&content).expect("Failed to parse YAML")
}

fn get_time(offset: i32) -> String {
    let utc_now = Utc::now();
    let tz_offset = FixedOffset::east_opt(offset * 3600).unwrap();
    utc_now
        .with_timezone(&tz_offset)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = read_config();
    let current_time = get_time(config.timezone);
     println!("Content-Type: text/html\n");  
    println!("<html lang=\"en\">");
    println!("<head>");
    println!("<title>Current time</title>");
    println!("</head>");
    println!("<body>");
    println!(
        "<h1>The Current Time in {}, {} is {}</h1></body></html>",
        config.city, config.country, current_time
    );
    println!("<a href='set_sa_time.cgi'>Switch South African Time</a></br>");
    println!("<a href='set_uk_time.cgi'>Switch United Kingdom Time</a>");
    println!("</body>");
    println!("</html>");

    Ok(())
}
