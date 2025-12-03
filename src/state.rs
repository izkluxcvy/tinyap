use dotenvy::dotenv;
use sqlx::SqlitePool;
use std::env;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub domain: String,
    pub tera: Tera,
    pub config: Config,
}

#[derive(Clone)]
pub struct Config {
    pub timezone: String,
    pub session_ttl_days: i64,
    pub session_max_per_user: i64,
    pub allow_signup: bool,
    pub max_timeline_notes: i64,
    pub max_note_chars: usize,
}

pub async fn init_state() -> AppState {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = SqlitePool::connect(&database_url).await.unwrap();
    let domain = env::var("DOMAIN").expect("DOMAIN must be set");

    let timezone = env::var("TIMEZONE").expect("TIMEZONE must be set");
    let session_ttl_days = env::var("SESSION_TTL_DAYS")
        .expect("SESSION_TTL_DAYS must be set")
        .parse::<i64>()
        .expect("SESSION_TTL_DAYS must be a valid number");
    let session_max_per_user = env::var("SESSION_MAX_PER_USER")
        .expect("SESSION_MAX_PER_USER must be set")
        .parse::<i64>()
        .expect("SESSION_MAX_PER_USER must be a valid number");
    let allow_signup = env::var("ALLOW_SIGNUP")
        .expect("ALLOW_SIGNUP must be set")
        .parse::<bool>()
        .expect("ALLOW_SIGNUP must be a valid boolean");
    let max_timeline_notes = env::var("MAX_TIMELINE_NOTES")
        .expect("MAX_TIMELINE_NOTES must be set")
        .parse::<i64>()
        .expect("MAX_TIMELINE_NOTES must be a valid number");
    let max_note_chars = env::var("MAX_NOTE_CHARS")
        .expect("MAX_NOTE_CHARS must be set")
        .parse::<usize>()
        .expect("MAX_NOTE_CHARS must be a valid number");

    let config = Config {
        timezone: timezone,
        session_ttl_days: session_ttl_days,
        session_max_per_user: session_max_per_user,
        allow_signup: allow_signup,
        max_timeline_notes: max_timeline_notes,
        max_note_chars: max_note_chars,
    };

    AppState {
        db_pool: db_pool,
        domain: domain,
        tera: Tera::new("templates/*").unwrap(),
        config: config,
    }
}

#[cfg(feature = "tls")]
pub fn cert_files() -> (String, String) {
    let cert_path = env::var("CERT_PATH").expect("CERT_PATH must be set");
    let key_path = env::var("KEY_PATH").expect("KEY_PATH must be set");
    (cert_path, key_path)
}

pub fn server_address() -> String {
    let host = env::var("HOST").expect("HOST must be set");
    let port = env::var("PORT").expect("PORT must be set");
    format!("{}:{}", host, port)
}
