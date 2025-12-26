use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
            .replace("\"", "")
            .replace("'", "")
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
    pub domain: String,
}

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
    let domain = conf.get("domain").expect("domain must be set").to_string();
    AppState {
        db_pool: db_pool,
        domain: domain,
    }
}
