use std::env;

pub fn redis_connection() -> redis::Connection {
    let host  = env::var("REDIS_URL").is_err();

    match redis::Client::open(host) {
        Err(e) => panic!("{:#?}", e),
        Ok(client) => client.get_connection().unwrap()
    }
}