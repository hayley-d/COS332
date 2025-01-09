use std::path::PathBuf;
use std::sync::Arc;

use redis::{AsyncCommands, Commands};
use tokio::fs;
use tokio::sync::Mutex;

/// Returns 7 if the redis connection is properly working
pub fn set_up_redis() -> redis::RedisResult<isize> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut connection = client.get_connection()?;
    // throw away the result, just to make sure it does not fail
    let _: () = connection.set("some_key", 7)?;
    // read the key and return it
    connection.get("some_key")
}

// caches a files content and returns the string
pub async fn read_and_cache_page(
    redis_connection: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    mut path: PathBuf,
) -> Vec<u8> {
    let content: String = match fs::read_to_string(path.clone()).await {
        Ok(content) => content,
        Err(_) => fs::read_to_string("static/404.html")
            .await
            .expect("404 Not Found"),
    };

    // set for 10 minuets
    let _: () = redis_connection
        .lock()
        .await
        .set_ex(path.pop(), &content, 600)
        .await
        .unwrap();

    return content.as_bytes().to_vec();
}

pub async fn get_cached_content(
    redis_connection: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    mut path: PathBuf,
) -> Option<Vec<u8>> {
    match redis_connection
        .lock()
        .await
        .get::<_, String>(path.pop())
        .await
    {
        Ok(content) => Some(content.as_bytes().to_vec()),
        Err(_) => None,
    }
}
