use std::error::Error;
use std::env;
use crate::redis_conn;
use crate::http_client;
use scraper::{Html, Selector};
use crate::operations::create;

pub fn execute() -> Result<(), Box<dyn Error>> {
    println!("Get list of years...");

    let link = env::var("LINK")?;

    let mut redis = redis_conn::redis_connection();

    let mut http =
        http_client::Client::new(link + "?pn=0", &mut redis);

    let mut years: Vec<String> = vec![];

    let body = http.body().unwrap();

    let document = Html::parse_document(&body);

    let selector = Selector::parse("select[name=year]").unwrap();
    let option = Selector::parse("option").unwrap();

    for element in document.select(&selector) {
        for option in element.select(&option) {
            years.push(option.inner_html());
        }
    }

    years.reverse();

    for year in &years {
        create(
            "get_page_count",
            &[("year".to_string(), year.to_string())]
        )?;
    }

    println!("...{}", &years.join(", "));

    Ok(())
}