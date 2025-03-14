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
    fs::write("static/backend.yaml", yaml).expect("Failed to write file");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let new_config: Config = Config {
        timezone: 0,
        country: "United Kingdom".to_string(),
        city: "London".to_string(),
    };

    write_config(&new_config);

    println!("Content-Type: text/html\n");  
    println!("<html lang=\"en\">");
    println!("<head>");
    println!("<meta http-equiv='refresh' content='0;url=show_time.cgi'>");
    println!("</head>");
    println!("<body>");
    println!("<h3>Switching to United Kingdom time</h3>");
    println!("</body>");
    println!("</html>");
    Ok(())
}
