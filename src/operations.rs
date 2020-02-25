use redis::Commands;
use crate::redis_conn;
use std::time::SystemTime;
use std::panic::resume_unwind;
use std::io::{Error, ErrorKind};

mod get_page;
mod get_page_count;
mod get_years;

pub fn timestamp() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    }
}

pub fn create(name: &str, args: &[(String, String)]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Create {}...", &name);

    let mut redis = redis_conn::redis_connection();

    let key: Vec<_> =
        args
            .iter()
            .map(move |pair| format!("{}:{}", pair.0, pair.1))
            .collect();

    let key = format!("{}||{}", name, key.join("|"));

    let _: () = redis.sadd("operations", key)?;

    println!("...created {} {:?}", &name, &args);

    Ok(())
}

pub fn  get() -> Result<String, Box<dyn std::error::Error>> {
    let mut redis = redis_conn::redis_connection();

    let operation: String = redis::transaction(&mut redis, &["operations"], |con, _pipe| {
        let count: usize = con.scard("operations")?;

        let operations: Vec<String> = con.srandmember_multiple("operations", count)?;

        for operation in operations {
            let res = match con.get(&operation) {
                Err(_) => 0,
                Ok(r) => r
            };

            if res != 1 {
                let _: () = con.set_ex(&operation, 1, 300)?;

                return Ok(Some(operation))
            }
        }

        Ok(Some("".to_string()))
    })?;

    if operation == "" {
        return Err(Box::new(Error::new(ErrorKind::Other, "Not found operation")))
    }

    Ok(operation)
}

pub fn execute(key: String) -> Result<(), Box<dyn std::error::Error>> {
    let values: Vec<&str> = key.split("||").collect();

    if values.len() == 0 {
        return Err(Box::new(Error::new(ErrorKind::Other, "Nullable operation")))
    }

    let operation= values[0];

    let values: Vec<&str> = {
        if values.len() > 1 {
            values[1].split("|").collect()
        } else {
            vec![""]
        }
    };

    let values: Vec<Vec<&str>> = if values[0].is_empty() {
        vec![vec![]]
    } else {
        values.iter().map(|&value| value.split(":").collect()).collect()
    };

    match operation {
        "get_years" => match get_years::execute() {
            Err(e) => println!("{:?}", e),
            _ => {}
        },
        "get_page_count" => {
            let mut year = "2010";

            for value in values {
                if value.len() > 1 && value[0] == "year" {
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

            for value in &values {
                if value.len() > 1 && value[0] == "page" {
                    page = value[1]
                }
            }

            let mut year = "2010";

            for value in values {
                if value.len() > 1 && value[0] == "year" {
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

    let _:() = redis.srem("operations", key)?;

    Ok(())
}