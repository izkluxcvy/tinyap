use crate::VERSION;
use crate::back::init;
use crate::web;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

fn web_routes() -> Router {
    Router::new().route("/", get(web::index::get))
}

pub async fn serve() {
    println!("[TinyAP version {}]", VERSION);

    let app = web_routes();

    let server_addr = init::server_address();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Server listening on http://{}", &server_addr);
    axum::serve(listener, app).await.unwrap();
}
