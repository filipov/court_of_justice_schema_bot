use std::error::Error;
use crate::{redis_conn, http_client};
use std::env;
use scraper::{Html, Selector};
use std::cmp::max;
use crate::operations::create;
use core::time;
use std::thread::sleep;

pub fn execute(page: String, year: String) -> Result<(), Box<dyn Error>> {
    println!("Get page {} of {} year...", &page, &year);

    let link = env::var("LINK").unwrap();

    let mut redis = redis_conn::redis_connection();

    let mut http =
        http_client::Client::new(
            format!("{}?pn={}&year={}", link, page, year),
            &mut redis
        );

    let body = http.body().unwrap();

    let document = Html::parse_document(&body);

    let selector = Selector::parse("table.decision_table").unwrap();
    let tr = Selector::parse("tr").unwrap();
    let td = Selector::parse("td").unwrap();
    let a = Selector::parse("a").unwrap();

    let rows = document
        .select(&selector).next().unwrap()
        .select(&tr);

    for row in rows {
        let cells =
            row.select(&td).collect::<Vec<_>>();

        if cells.len() > 1 {
            let number = &cells[1].inner_html();

            let link =
                cells[cells.len() - 1].select(&a).next().unwrap();

            let url = link.value().attr("href").unwrap();

            create("get_document", &[
                ("number".to_string(), number.to_string()),
                ("link".to_string(), url.to_string())
            ])?;

            sleep(time::Duration::from_millis(1));
        }
    }

    println!("... {:#?} page", page);

    Ok(())
}