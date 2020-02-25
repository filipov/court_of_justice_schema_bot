use redis::Commands;
use crate::redis_conn;
use std::error::Error;
use std::time::SystemTime;
use std::panic::resume_unwind;
use std::io::ErrorKind;

mod get_page;
mod get_page_count;
mod get_years;

pub fn timestamp() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    }
}

pub fn create(name: &str, args: &[(String, String)]) -> Result<(), Box<dyn Error>> {
    println!("Create {}...", &name);

    let mut redis = redis_conn::redis_connection();

    let key: Vec<_> =
        args
            .iter()
            .map(move |pair| format!("{}:{}", pair.0, pair.1))
            .collect();

    let key = format!("{}||{}", name, key.join("|"));

    let _: () = redis.sadd("operations", key)?;

    /////////////////////////////////////////////////////
    // let key = format!("operation:{}", timestamp());
    //
    // let mut args = args.to_owned();
    //
    // args.push(("name".to_string(), name.to_string()));
    //
    // redis.hset_multiple(&key, &args)?;

    println!("...created {} {:?}", &name, &args);

    Ok(())
}

pub fn  get() -> Result<String, Box<dyn Error>> {
    let mut redis = redis_conn::redis_connection();

    let operation: String = redis::transaction(&mut redis, &["operations"], |con, _pipe| {
        let count: usize = con.scard("operations")?;

        let operations: Vec<String> = con.srandmember_multiple("operations", count)?;

        for operation in operations {
            if con.get(&operation)? != 1 {
                con.set_ex(&operation, 1, 300);

                return Ok(Some(operation))
            }
        }

        Err(Error::new(ErrorKind::Other, "Not found operation"))
    })?;

    Ok(operation)

    /////////////////////////////////////////////////////
    // let mut keys: std::vec::Vec<String> = match redis.keys("operation:*") {
    //     Ok(keys) => keys,
    //     Err(e) => {
    //         println!("{:?}", e);
    //         Vec::new()
    //     }
    // };
    //
    // keys.sort();
    //
    // if keys.is_empty() {
    //     match create("get_years", &Vec::new()) {
    //         Ok(_) => {},
    //         Err(e) => println!("{:?}", e)
    //     };
    //
    //     return "".to_string();
    // }
    //
    // match redis::transaction(&mut redis, &keys, |con, _pipe| {
    //     let mut operation: String = "".to_string();
    //
    //     for key in &keys {
    //         let used = match con.get(format!("wip:{}", key)) {
    //             Ok(result) => result,
    //             Err(_) => 0
    //         };
    //
    //         if used == 0 {
    //             operation = key.to_owned();
    //
    //             break;
    //         }
    //     }
    //
    //     if operation.is_empty() {
    //         match create("get_years", &Vec::new()) {
    //             Ok(_) => {},
    //             Err(e) => println!("{:?}", e)
    //         };
    //     }
    //
    //     con.set_ex(format!("wip:{}", operation), 1, 600)?;
    //
    //     Ok(Some(operation))
    // }) {
    //     Ok(res) => res,
    //     Err(e) => {
    //         println!("{:?}", e);
    //         "".to_string()
    //     }
    // }
}

pub fn execute(key: String) {
    let (operation, values): (&str, &str) = key.split("||").collect();

    let values: Vec<&str> = values.split("|").collect();

    let values: Vec<Vec<&str>> = values.iter().map(|&value| value.split(":")).collect();

    match operation.as_str() {
        "get_years" => match get_years::execute() {
            Err(e) => println!("{:?}", e),
            _ => {}
        },
        "get_page_count" => {
            let mut year = "2010";

            for value in values {
                if value[0] == "year" {
                    year = value[1]
                }
            }

            match get_page_count::execute(year.to_string()) {
                Err(e) => println!("{:?}", e),
                _ => {}
            }
        },
        "get_page" => {
            let mut page = "0";

            for value in values {
                if value[0] == "page" {
                    page = value[1]
                }
            }

            let mut year = "2010";

            for value in values {
                if value[0] == "year" {
                    year = value[1]
                }
            }

            match get_page::execute(page.to_string(), year.to_string()) {
                Err(e) => println!("{:?}", e),
                _ => {}
            }
        },
        e => println!("{:?}", e)
    };

    let mut redis = redis_conn::redis_connection();

    redis.srem("operations", key);
}