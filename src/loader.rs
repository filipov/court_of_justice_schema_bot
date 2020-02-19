use scraper::{Html, Selector};
use std::cmp::max;
use crate::http_client::Client;

pub fn get_years(http: &mut Client) -> Vec<String> {
    println!("Get list of years...");

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

    println!("...{}", &years.join(", "));

    years
}

pub fn get_year_page_count(http: &mut Client, year: &str) -> u16 {
    println!("Get page count of {} year...", &year);

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

    real_count
}