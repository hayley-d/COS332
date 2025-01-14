use std::path::PathBuf;

use log::{error, info};
use redis::Commands;
use tokio::fs;
use tokio::process::{Child, Command};

pub async fn start_redis_server() -> Child {
    return match Command::new("redis-server").spawn() {
        Ok(child) => {
            println!("Redis server started");
            info!("Redis server started");
            child
        }
        _ => {
            eprintln!("Problem starting redis server");
            error!(target:"error_logger","Failed to start Redis server");
            std::process::exit(1);
        }
    };
}

pub async fn stop_redis_server(mut redis_child: Child) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = redis_child.kill().await {
        eprintln!("Failed to stop redis server");
        return Err(Box::new(e));
    }

    println!("Redis server stopped");
    info!("Redis server stopped");

    Ok(())
}

/// Returns a connection to the redis instance
pub fn set_up_redis() -> Result<redis::Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    return Ok(client.get_connection()?);
}

// caches a files content and returns the string
pub async fn read_and_cache_page(
    redis_connection: &mut redis::Connection,
    path: &PathBuf,
    route_name: &str,
) -> Vec<u8> {
    let content: String = match fs::read_to_string(path.clone()).await {
        Ok(content) => content,
        Err(_) => fs::read_to_string("static/404.html")
            .await
            .expect("404 Not Found"),
    };

    // set for 10 minuets
    let _: () = redis_connection.set_ex(route_name, &content, 600).unwrap();
    return content.as_bytes().to_vec();
}

pub async fn get_cached_content(
    redis_connection: &mut redis::Connection,
    route_name: &str,
) -> Option<Vec<u8>> {
    match redis_connection.get::<_, String>(route_name) {
        Ok(content) => Some(content.as_bytes().to_vec()),
        Err(_) => None,
    }
}
