use core::time;
use std::thread::sleep;
use redis::{Connection, Commands};
use std::error::Error;
use std::time::SystemTime;

const MAX: u8 = 6;

const TIMEOUT: u64 = 6000;

fn timestamp() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    }
}

pub struct Client<'a> {
    errors_count: u8,
    pub link: String,
    pub text: String,

    is_loaded: bool,

    redis: &'a mut Connection
}

impl<'a> Client<'a> {
    pub fn new(link: String, redis: &'a mut Connection) -> Client {
        Client {
            errors_count: 0,
            link,
            text: "".to_string(),

            is_loaded: false,

            redis
        }
    }

    // Создание объекта запроса с N возможным количеством ошибок
    fn create_request(&mut self) -> nano_get::Request {
        match nano_get::Request::default_get_request(self.link.to_owned()) {
            Err(e) => {
                if self.errors_count < MAX {
                    println!("{:#?}", e);
                    self.errors_count = self.errors_count + 1;
                    self.create_request()
                } else {
                    panic!("{:#?}", e);
                }
            },
            Ok(request) => {
                self.errors_count = 0;
                request
            }
        }
    }

    // Вызов запроса
    fn execute(&mut self) -> nano_get::Response {
        match self.create_request().execute() {
            Err(e) => {
                if self.errors_count < MAX {
                    println!("{:#?}", e);
                    self.errors_count = self.errors_count + 1;
                    sleep(time::Duration::from_millis(TIMEOUT));
                    self.execute()
                } else {
                    panic!("{:#?}", e);
                }
            },
            Ok(response) => {
                self.errors_count = 0;
                response
            }
        }
    }

    // Запрос тела по ссылке
    pub fn body(&mut self) -> Result<String, Box<dyn Error>> {
        if self.is_loaded {
            Ok(self.text.to_owned())
        } else {
            let mut status = 0;

            while status != 200 && status != 404 {
                let resp = self.execute();

                status = match resp.get_status_code() {
                    Some(code) => code,
                    _ => 600
                };

                self.text = resp.body;

                // Записываем что-то где-то
                {
                    let _ : () = self.redis.rpush(
                        self.link.to_owned(),
                        format!(
                            "{{ \"status\": \"{}\", \"at\": \"{}\" }}",
                            status.to_string(), timestamp()
                        )
                    ).unwrap();
                }

                sleep(time::Duration::from_millis(TIMEOUT));
            }

            Ok(self.text.to_owned())
        }
    }
}