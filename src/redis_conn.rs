pub fn redis_connection() -> redis::Connection {
    match redis::Client::open("redis://127.0.0.1:32768/0") {
        Err(e) => panic!("{:#?}", e),
        Ok(client) => client.get_connection().unwrap()
    }
}