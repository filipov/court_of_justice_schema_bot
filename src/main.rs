use redis::Commands;
use std::{env, thread, process};

mod http_client;
mod operations;
mod redis_conn;

fn set_name() {
    let link = match env::var("LINK") {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            process::exit(1)
        }
    };

    let mut redis = redis_conn::redis_connection();

    match redis.set("name", link){
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            process::exit(1)
        }
    };
}

fn main() {
    set_name();

    let threads_count: i32 = match env::var("THREAD_COUNT") {
        Ok(r) => r.parse().unwrap(),
        Err(_) => 1
    };

    let mut threads = vec![];

    for _i in 0..threads_count {
        threads.push(thread::spawn( move || {
            loop {
                match operations::get().as_str() {
                    "" => {}
                    operation =>
                        operations::execute(operation.to_string())
                }
            }
        }));
    }

    for thr in threads {
        let _ = thr.join();
    }
}