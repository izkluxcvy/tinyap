use dotenvy::dotenv;
use std::env;

pub fn server_address() -> String {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST must be set");
    let port = env::var("PORT").expect("PORT must be set");
    format!("{}:{}", host, port)
}
