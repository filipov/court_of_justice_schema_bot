use std::error::Error;
use crate::{redis_conn, http_client};
use std::env;
use scraper::{Html, Selector};
use std::cmp::max;
use crate::operations::create;
use core::time;
use std::thread::sleep;

pub fn execute(year: String) -> Result<(), Box<dyn Error>> {
    println!("Get page count of {} year...", &year);

    let link = env::var("LINK").unwrap();

    let mut redis = redis_conn::redis_connection();

    let mut http =
        http_client::Client::new(
            format!("{}?year={}", link, year),
            &mut redis
        );

    let body = http.body().unwrap();

    let document = Html::parse_document(&body);

    let selector = Selector::parse("div#pager_wrapper").unwrap();
    let a = Selector::parse("a").unwrap();

    let links = document
        .select(&selector).next().unwrap()
        .select(&a);

    let mut real_count = 0;

    for link in links {
        let href = &link.value().attr("href").unwrap()[1..];

        let count_query: Vec<&str> = href.split("&").collect();

        let count: u16 = count_query[0][3..].parse::<u16>().unwrap();

        real_count = max(real_count, count)
    }

    real_count += 1;

    println!("... {:#?} pages", &real_count);

    for page in 0..real_count {
        create(
            "get_page",
            &[
                ("year".to_string(), year.to_string()),
                ("page".to_string(), page.to_string())
            ]
        );

        sleep(time::Duration::from_millis(1));
    }

    Ok(())
}