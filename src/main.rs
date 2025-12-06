mod ap;
mod auth;
mod boost;
mod delete;
mod follow;
mod like;
mod note;
mod state;
mod user;
mod web;

use axum::{
    Router,
    routing::{get, get_service, post},
};
use tower_http::services::ServeFile;

pub async fn routing() -> Router {
    let app_state = state::init_state().await;
    Router::new()
        // Web UIs
        .route("/", get(web::index::page))
        .route("/signup", get(web::signup::page).post(user::signup))
        .route("/login", get(web::login::page).post(user::login))
        .route("/logout", get(web::logout::page).post(user::logout))
        .route("/home", get(web::home::page))
        .route("/notifications", get(web::notifications::page))
        .route("/new", get(web::new::page).post(note::create_note))
        .route("/local", get(web::local::page))
        .route("/federated", get(web::federated::page))
        .route("/@{username}", get(web::user::page))
        .route("/@{username}/following", get(web::follows::following))
        .route("/@{username}/followers", get(web::follows::followers))
        .route("/@{username}/{uuid}", get(web::note::page))
        .route("/@{username}/{uuid}/like", post(like::like))
        .route("/@{username}/{uuid}/unlike", post(like::unlike))
        .route("/@{username}/{uuid}/boost", post(boost::boost))
        .route("/@{username}/{uuid}/unboost", post(boost::unboost))
        .route("/@{username}/{uuid}/delete", post(delete::note))
        .route("/@{username}/follow", post(follow::follow))
        .route("/@{username}/unfollow", post(follow::unfollow))
        .route(
            "/profile",
            get(web::profile::page).post(user::update_profile),
        )
        .route("/change_password", post(user::update_password))
        // ActivityPub endpoints
        .route("/users/{username}", get(ap::actor::api))
        .route("/.well-known/webfinger", get(ap::webfinger::api))
        .route("/.well-known/nodeinfo", get(ap::nodeinfo::well_known))
        .route("/nodeinfo/2.1", get(ap::nodeinfo::nodeinfo))
        .route("/notes/{uuid}", get(ap::note::api))
        .route("/users/{username}/inbox", post(ap::inbox::api))
        .route("/users/{username}/outbox", get(ap::outbox::api))
        // Static files
        .route(
            "/style.css",
            get_service(ServeFile::new("static/style.css")),
        )
        .with_state(app_state)
}

// HTTP without TLS
#[cfg(not(feature = "tls"))]
use tokio::net::TcpListener;
#[cfg(not(feature = "tls"))]
#[tokio::main]
async fn main() {
    let app = routing().await;
    println!("Server listening on http://{}", state::server_address());
    let listener = TcpListener::bind(&state::server_address()).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

// HTTPS with TLS
#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use std::net::SocketAddr;
#[cfg(feature = "tls")]
#[tokio::main]
async fn main() {
    let app = routing().await;

    let (cert_path, key_path) = state::cert_files();
    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .unwrap();

    println!("Server listening on https://{}", state::server_address());
    let addr = SocketAddr::from(state::server_address().parse::<SocketAddr>().unwrap());
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
