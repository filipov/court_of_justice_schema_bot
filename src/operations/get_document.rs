//+ Номер участка МС:	347
//+ Номер первичного документа:
//+ Дата регистрации:
// Наименование документа:	Исковое заявление
//+ Истец:
//+ Ответчик:
//+ Судья:	Арсеньева М.Н.
//+ Текущее состояние:	Возвращено
// Документ:
// История состояний:
    // Документ-основание
    // Состояние
    // Дата
//+ УИД:

use std::error::Error;
use std::env;
use url::Url;
use crate::{redis_conn, http_client};
use scraper::{Html, Selector};
use regex::Regex;
use redis::Commands;

pub fn execute(number: String, path: String) -> Result<(), Box<dyn Error>> {
    println!("Get document {}...", number);

    let link = {
        let link = env::var("LINK").unwrap();

        let url = Url::parse(&link)?;

        let url = url.join(&path)?;

        url.as_str().to_owned()
    };

    let mut redis = redis_conn::redis_connection();

    let mut http = http_client::Client::new(link.to_owned(), &mut redis);

    let body = http.body()?;

    let document = Html::parse_document(&body);

    let selector = Selector::parse("table").unwrap();
    let tr = Selector::parse("tr").unwrap();
    let td = Selector::parse("td").unwrap();
    let strong = Selector::parse("strong").unwrap();

    let tables = document.select(&selector);

    for table in tables {
        let cells =
            table.select(&td).collect::<Vec<_>>();

        if cells.len() > 0 {
            let strongs =
                cells[0].select(&strong).collect::<Vec<_>>();

            if strongs.len() > 0 {
                if strongs[0].inner_html().to_owned() == "Номер участка МС:" {
                    let rows = table.select(&tr);

                    let mut items = vec![];

                    for row in rows {
                        let cells =
                            row.select(&td).collect::<Vec<_>>();

                        let re = Regex::new(r"<[^>]*>(?P<text>.*)</[^>]*>", ).unwrap();

                        if cells.len() > 1 {
                            let cell = re.replace_all(&cells[0].inner_html(), "$text").trim().to_string();

                            let field = match cell.clone().as_str() {
                                "Номер участка МС:" => "courtNumber",
                                "Текущее состояние:" => "currentState",
                                "Истец:" => "plaintiff",
                                "Судья:" => "justice",
                                "Номер первичного документа:" => "numberFirstDoc",
                                "Номер дела:" => "numberDoc",
                                "Предмет спора:" => "subject",
                                "Дата регистрации дела:" => "registeredAt",
                                "Дата регистрации:" => "registeredAt",
                                "Ответчик:" => "defendant",
                                "УИД:" => "uid",
                                field => field
                            }.to_string();

                            let value = re.replace_all(&cells[1].inner_html(), "$text").trim().to_owned();

                            items.push((field.clone(), value.clone()));

                            println!("{:?}", (field, value))
                        }
                    }

                    items.push(("path".to_string(), path.clone()));

                    let mut redis = redis_conn::redis_connection();

                    redis.hset_multiple(&number, &items)?;

                    redis.sadd("documents", &number)?
                }
            }
        }
    }

    println!("...document {} put", number);

    Ok(())
}