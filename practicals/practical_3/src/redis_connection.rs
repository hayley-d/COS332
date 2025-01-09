use std::path::PathBuf;

use redis::Commands;
use tokio::fs;

/// Returns a connection to the redis instance
pub fn set_up_redis() -> Result<redis::Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    return Ok(client.get_connection()?);
}

// caches a files content and returns the string
pub async fn read_and_cache_page(
    redis_connection: &mut redis::Connection,
    mut path: PathBuf,
) -> Vec<u8> {
    let content: String = match fs::read_to_string(path.clone()).await {
        Ok(content) => content,
        Err(_) => fs::read_to_string("static/404.html")
            .await
            .expect("404 Not Found"),
    };

    // set for 10 minuets
    let _: () = redis_connection.set_ex(path.pop(), &content, 600).unwrap();
    return content.as_bytes().to_vec();
}

pub async fn get_cached_content(
    redis_connection: &mut redis::Connection,
    mut path: PathBuf,
) -> Option<Vec<u8>> {
    match redis_connection.get::<_, String>(path.pop()) {
        Ok(content) => Some(content.as_bytes().to_vec()),
        Err(_) => None,
    }
}
