use crate::VERSION;
use crate::back::init;
use crate::web;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

async fn web_routes() -> Router {
    Router::new()
        .route("/", get(web::index::get))
        .route("/signup", get(web::signup::get).post(web::signup::post))
        .route("/login", get(web::login::get).post(web::login::post))
        .with_state(init::create_app_state().await)
}

pub async fn serve() {
    println!("[TinyAP version {}]", VERSION);

    let app = web_routes().await;

    let server_addr = init::server_address();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Server listening on http://{}", &server_addr);
    axum::serve(listener, app).await.unwrap();
}
