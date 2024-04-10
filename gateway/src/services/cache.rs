use std::env;

use redis::Client;

pub fn init() -> Client {
    let redis_url = env::var("REDIS_URL").expect("env::REDIS_URL is missing");

    redis::Client::open(redis_url).expect("Error to init redis client")
}
