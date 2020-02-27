use core::time;
use std::thread::sleep;
use redis::{Connection, Commands};
use std::error::Error;
use std::time::SystemTime;
use encoding::{DecoderTrap, Encoding};
use encoding::all::{WINDOWS_1251};
use curl::easy::Easy;

const TIMEOUT: u64 = 6000;

fn timestamp() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    }
}

pub struct Client<'a> {
    pub link: String,
    pub text: String,

    is_loaded: bool,

    redis: &'a mut Connection
}

impl<'a> Client<'a> {
    pub fn new(link: String, redis: &'a mut Connection) -> Client {
        Client {
            link,
            text: "".to_string(),

            is_loaded: false,

            redis
        }
    }

    // Создание объекта запроса с N возможным количеством ошибок
    fn create_request(&mut self) -> curl::easy::Easy {
        let mut easy = Easy::new();

        match easy.url(&self.link) {
            Err(e) => panic!("{:?}", e),
            Ok(e) => e
        };

        easy
    }

    // Вызов запроса
    fn execute(&mut self) -> Result<(Vec<u8>, u32), Box<dyn Error>> {
        let mut data = Vec::new();

        let mut code = 0;

        {
            let mut easy = self.create_request();

            {
                let mut transfer = easy.transfer();

                transfer.write_function(|body| {
                    for slice in body {
                        data.push(slice.to_owned())
                    }

                    Ok(body.len())
                })?;

                transfer.perform()?;
            }

            code = match easy.response_code() {
                Ok(code) => code,
                Err(_) => 600
            };
        }

        Ok((data, code))
    }

    // Запрос тела по ссылке
    pub fn body(&mut self) -> Result<String, Box<dyn Error>> {
        if self.is_loaded {
            Ok(self.text.to_owned())
        } else {
            let mut status = 0;

            while status != 200 && status != 404 {
                let result = self.execute()?;

                let data = result.0;

                status = result.1;

                self.text = match WINDOWS_1251.decode(&data, DecoderTrap::Replace) {
                    Ok(r) => r,
                    Err(e) => { println!("{:?}", e); "".to_string() }
                };

                // Записываем что-то где-то
                {
                    let _: () = match self.redis.rpush(
                        self.link.to_owned(),
                        format!(
                            "{{ \"status\": {}, \"at\": {} }}",
                            status.to_string(), timestamp()
                        )
                    ) {
                        Ok(r) => r,
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    };
                }

                sleep(time::Duration::from_millis(TIMEOUT));
            }

            Ok(self.text.to_owned())
        }
    }
}