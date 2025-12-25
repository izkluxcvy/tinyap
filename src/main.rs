mod init;
mod web;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn web_routes() -> Router {
    Router::new().route("/", get(web::index::get))
}

#[tokio::main]
async fn main() {
    println!("[TinyAP version {}]", VERSION);

    let app = web_routes();

    let server_addr = init::server_address();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Server listening on http://{}", &server_addr);
    axum::serve(listener, app).await.unwrap();
}
