use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tera::Tera;

fn load_config() -> HashMap<String, String> {
    let file = File::open("config.yaml").expect("Failed to open config.yaml");
    let reader = BufReader::new(file);

    let mut conf = HashMap::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.splitn(2, ":").collect();

        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim().to_string();
        let value = parts[1]
            .trim()
            .trim_start_matches("\"")
            .trim_end_matches("\"")
            .trim_start_matches("'")
            .trim_end_matches("'")
            .to_string();
        conf.insert(key, value);
    }
    conf
}

pub fn server_address() -> String {
    let conf = load_config();
    let host = conf.get("host").expect("host must be set");
    let port = conf.get("port").expect("port must be set");
    format!("{}:{}", host, port)
}

#[derive(Clone)]
pub struct AppState {
    #[cfg(feature = "sqlite")]
    pub db_pool: sqlx::SqlitePool,
    #[cfg(feature = "postgres")]
    pub db_pool: sqlx::PgPool,
    pub tera: Tera,
    pub domain: String,
    pub metadata: Metadata,
    pub config: Config,
}

#[derive(Clone)]
pub struct Metadata {
    pub instance_name: String,
}

#[derive(Clone)]
pub struct Config {
    pub allow_signup: bool,
    pub session_ttl_days: i64,
    pub max_sessions_per_user: i64,
}

// Compile-time checks for database feature flags
#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!("sqlite or postgres feature must be enabled");

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("sqlite and postgres features must be enabled exclusively");

#[cfg(feature = "sqlite")]
async fn create_db_pool(conf: &HashMap<String, String>) -> sqlx::SqlitePool {
    let database_url = conf.get("database_url").expect("database_url must be set");
    println!("Connecting to SQLite database...");
    sqlx::SqlitePool::connect(database_url).await.unwrap()
}

#[cfg(feature = "postgres")]
async fn create_db_pool(conf: &HashMap<String, String>) -> sqlx::PgPool {
    let database_url = conf.get("database_url").expect("database_url must be set");
    println!("Connecting to PostgreSQL database...");
    sqlx::PgPool::connect(database_url).await.unwrap()
}

pub async fn create_app_state() -> AppState {
    let conf = load_config();

    let db_pool = create_db_pool(&conf).await;

    let tera = Tera::new("templates/**/*").unwrap();

    let domain = conf.get("domain").expect("domain must be set").to_string();

    let instance_name = conf
        .get("instance_name")
        .expect("instance_name must be set")
        .to_string();

    let allow_signup = conf
        .get("allow_signup")
        .expect("allow_signup must be set")
        .parse::<bool>()
        .expect("allow_signup must be a boolean");

    let session_ttl_days = conf
        .get("session_ttl_days")
        .expect("session_ttl_days must be set")
        .parse::<i64>()
        .expect("session_ttl_days must be an integer");

    let max_sessions_per_user = conf
        .get("max_sessions_per_user")
        .expect("max_sessions_per_user must be set")
        .parse::<i64>()
        .expect("max_sessions_per_user must be an integer");

    AppState {
        db_pool: db_pool,
        tera: tera,
        domain: domain,
        metadata: Metadata {
            instance_name: instance_name,
        },
        config: Config {
            allow_signup: allow_signup,
            session_ttl_days: session_ttl_days,
            max_sessions_per_user: max_sessions_per_user,
        },
    }
}
