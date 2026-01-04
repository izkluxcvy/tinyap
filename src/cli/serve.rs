use crate::VERSION;
use crate::activitypub as ap;
use crate::back::init;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

async fn activitypub_routes() -> Router<init::AppState> {
    Router::new()
        .route("/.well-known/webfinger", get(ap::webfinger::get))
        .route("/.well-known/nodeinfo", get(ap::nodeinfo::get_well_known))
        .route("/nodeinfo/2.1", get(ap::nodeinfo::get_nodeinfo))
        .route("/users/{username}", get(ap::actor::get))
        .route("/users/{username}/inbox", post(ap::inbox::post))
        .route("/inbox", post(ap::inbox::post))
        .route("/notes/{id}", get(ap::note::get))
}

#[cfg(feature = "web")]
async fn web_routes() -> Router<init::AppState> {
    use crate::web;
    use tower_http::services::ServeDir;
    Router::new()
        .route("/", get(web::index::get))
        .route("/signup", get(web::signup::get).post(web::signup::post))
        .route("/login", get(web::login::get).post(web::login::post))
        .route("/logout", get(web::logout::get).post(web::logout::post))
        .route(
            "/profile",
            get(web::profile::get).post(web::profile::post_profile),
        )
        .route("/change_password", post(web::profile::post_password))
        .route("/@{username}", get(web::user::get))
        .route("/@{username}/follow", post(web::follow::post_follow))
        .route("/@{username}/unfollow", post(web::follow::post_unfollow))
        .route("/@{username}/following", get(web::following::get_following))
        .route("/@{username}/followers", get(web::following::get_followers))
        .route("/new", get(web::new::get).post(web::new::post))
        .route("/@{username}/{id}", get(web::note::get))
        .route("/@{username}/{id}/delete", post(web::delete::post))
        .route("/@{username}/{id}/like", post(web::like::post_like))
        .route("/@{username}/{id}/unlike", post(web::like::post_unlike))
        .route("/@{username}/{id}/boost", post(web::boost::post_boost))
        .route("/@{username}/{id}/unboost", post(web::boost::post_unboost))
        .route("/notifications", get(web::notifications::get))
        .route("/home", get(web::timeline::get_home))
        .route("/local", get(web::timeline::get_local))
        .route("/federated", get(web::timeline::get_federated))
        .route("/search", get(web::search::get).post(web::search::post))
        .nest_service("/static", ServeDir::new("static"))
}

#[cfg(feature = "api")]
async fn api_routes() -> Router<init::AppState> {
    use crate::api;
    use axum::routing::delete;
    use tower_http::cors::{Any, CorsLayer};
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/instance", get(api::instance::get_v1))
        .route("/api/v2/instance", get(api::instance::get_v2))
        .route("/api/v1/apps", post(api::oauth::apps::post))
        .route(
            "/oauth/authorize",
            get(api::oauth::authorize::get).post(api::oauth::authorize::post),
        )
        .route(
            "/oauth/authorize/",
            get(api::oauth::authorize::get).post(api::oauth::authorize::post),
        )
        .route("/oauth/token", post(api::oauth::token::post))
        .route("/api/v1/accounts/{username}", get(api::accounts::get))
        .route(
            "/api/v1/accounts/{username}/statuses",
            get(api::accounts::get_statuses),
        )
        .route(
            "/api/v1/accounts/verify_credentials",
            get(api::verify_credentials::get),
        )
        .route("/api/v1/statuses", post(api::statuses::post))
        .route("/api/v1/statuses/{id}", get(api::statuses::get))
        .route("/api/v1/statuses/{id}", delete(api::statuses::delete))
        .route("/api/v1/statuses/{id}/context", get(api::context::get))
        .route(
            "/api/v1/statuses/{id}/favourite",
            post(api::favourite::post_favourite),
        )
        .route(
            "/api/v1/statuses/{id}/unfavourite",
            post(api::favourite::post_unfavourite),
        )
        .route(
            "/api/v1/statuses/{id}/reblog",
            post(api::reblog::post_reblog),
        )
        .route(
            "/api/v1/statuses/{id}/unreblog",
            post(api::reblog::post_unreblog),
        )
        .route("/api/v1/timelines/home", get(api::timeline::get_home))
        .route("/api/v1/timelines/public", get(api::timeline::get_public))
        .route("/api/v1/notifications", get(api::notifications::get))
        .route("/api/v1/search", get(api::search::get))
        .route("/api/v2/search", get(api::search::get))
        .layer(cors)
}

pub async fn serve() {
    println!("[TinyAP version {}]", VERSION);

    let state = init::create_app_state().await;

    let app = activitypub_routes().await;
    #[cfg(feature = "web")]
    let app = app.merge(web_routes().await);
    #[cfg(feature = "api")]
    let app = app.merge(api_routes().await);

    let app = app.with_state(state);

    let server_addr = init::server_address();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Server listening on http://{}", &server_addr);
    axum::serve(listener, app).await.unwrap();
}
