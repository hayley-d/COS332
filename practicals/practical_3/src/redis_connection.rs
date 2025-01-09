use redis::Commands;

pub fn set_up_redis() -> redis::RedisResult<isize> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut connection = client.get_connection()?;
    // throw away the result, just to make sure it does not fail
    let _: () = connection.set("some_key", 7)?;
    // read the key and return it
    connection.get("some_key")
}
