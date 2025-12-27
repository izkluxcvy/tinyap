use crate::VERSION;
use crate::activitypub as ap;
use crate::back::init;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

async fn activitypub_routes() -> Router<init::AppState> {
    Router::new().route("/.well-known/webfinger", get(ap::webfinger::get))
}

#[cfg(feature = "web")]
async fn web_routes() -> Router<init::AppState> {
    use crate::web;
    Router::new()
        .route("/", get(web::index::get))
        .route("/signup", get(web::signup::get).post(web::signup::post))
        .route("/login", get(web::login::get).post(web::login::post))
        .route("/new", get(web::new::get).post(web::new::post))
}

pub async fn serve() {
    println!("[TinyAP version {}]", VERSION);

    let state = init::create_app_state().await;

    let app = activitypub_routes().await;
    #[cfg(feature = "web")]
    let app = app.merge(web_routes().await);

    let app = app.with_state(state);

    let server_addr = init::server_address();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Server listening on http://{}", &server_addr);
    axum::serve(listener, app).await.unwrap();
}
