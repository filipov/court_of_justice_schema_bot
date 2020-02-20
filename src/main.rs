use crate::loader::{get_years, get_year_page_count};
use redis::Commands;
use std::collections::HashMap;
use std::env;

mod http_client;
mod loader;
mod redis_conn;

fn main() {
    let link = env::var("LINK").unwrap(); //"http://mos-sud.ru/services/caseinfo/caseinfocs/";

    let mut redis = redis_conn::redis_connection();

    let _ : () = redis.set("name", link.clone()).unwrap();

    let mut redis_http = redis_conn::redis_connection();

    let mut http =
        http_client::Client::new(
            link.to_owned() + "?pn=0",
            &mut redis_http
        );

    loop {
        // Get years list
        let years = get_years(&mut http);

        // And update this
        for year in years.clone() {
            let _ : () = redis.sadd("years", year).unwrap();
        }

        let mut pages: HashMap<String, u16> = HashMap::new();

        for year in years.clone() {
            let mut redis_http = redis_conn::redis_connection();

            let mut http =
                http_client::Client::new(
                    link.to_owned() + &format!("?year={}", year.to_owned()),
                    &mut redis_http
                );

            pages.insert((&year).to_string(), get_year_page_count(&mut http, &year));
        }

        for year in years.clone() {
            let _ : () = redis.rpush(format!("years:{}", &year), pages[&year].clone()).unwrap();
        }
    };
}