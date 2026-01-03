use crate::VERSION;

use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tera::Tera;
use tokio::sync::Semaphore;

fn load_config() -> HashMap<String, String> {
    let file = File::open("config.yaml").expect("Failed to open config.yaml");
    let reader = BufReader::new(file);

    let mut conf = HashMap::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ":").collect();

        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim().to_string();
        let value = parts[1]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
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
    pub db_pool: sqlx::SqlitePool,
    pub tera: Tera,
    pub deliver_queue: Arc<Semaphore>,
    pub http_client: Client,
    pub domain: String,
    pub re: Re,
    pub metadata: Metadata,
    pub config: Config,
    #[cfg(feature = "web")]
    pub web_config: WebConfig,
}

#[derive(Clone)]
pub struct Re {
    pub br: Regex,
    pub tag: Regex,
    pub link: Regex,
}

#[derive(Clone)]
pub struct Metadata {
    pub instance_name: String,
    pub instance_description: String,
}

#[derive(Clone)]
pub struct Config {
    pub max_note_chars: usize,
    pub token_ttl_days: i64,
}

#[cfg(feature = "web")]
#[derive(Clone)]
pub struct WebConfig {
    pub allow_signup: bool,
    pub max_sessions_per_user: i64,
    pub max_timeline_items: i64,
    pub timezone: String,
}

async fn create_db_pool(conf: &HashMap<String, String>) -> sqlx::SqlitePool {
    let database_url = conf.get("database_url").expect("database_url must be set");
    println!("Connecting to SQLite database...");
    sqlx::SqlitePool::connect(database_url).await.unwrap()
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

    let instance_description = conf
        .get("instance_description")
        .expect("instance_description must be set")
        .to_string();

    #[cfg(feature = "web")]
    let allow_signup = conf
        .get("allow_signup")
        .expect("allow_signup must be set")
        .parse::<bool>()
        .expect("allow_signup must be a boolean");

    #[cfg(feature = "web")]
    let max_sessions_per_user = conf
        .get("max_sessions_per_user")
        .expect("max_sessions_per_user must be set")
        .parse::<i64>()
        .expect("max_sessions_per_user must be an integer");

    let max_note_chars = conf
        .get("max_note_chars")
        .expect("max_note_chars must be set")
        .parse::<usize>()
        .expect("max_note_chars must be an integer");

    let token_ttl_days = conf
        .get("token_ttl_days")
        .expect("token_ttl_days must be set")
        .parse::<i64>()
        .expect("token_ttl_days must be an integer");

    let deliver_queue_size = conf
        .get("deliver_queue_size")
        .expect("deliver_queue_size must be set")
        .parse::<usize>()
        .expect("deliver_queue_size must be an integer");

    let http_client = Client::builder()
        .user_agent(&format!("TinyAP/{}", VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    #[cfg(feature = "web")]
    let timezone = conf
        .get("timezone")
        .expect("timezone must be set")
        .to_string();

    #[cfg(feature = "web")]
    let max_timeline_items = conf
        .get("max_timeline_items")
        .expect("max_timeline_items must be set")
        .parse::<i64>()
        .expect("max_timeline_items must be an integer");

    AppState {
        db_pool: db_pool,
        tera: tera,
        deliver_queue: Arc::new(Semaphore::new(deliver_queue_size)),
        http_client: http_client,
        domain: domain,
        re: Re {
            br: Regex::new(r"(?i)<br\s*/?>").unwrap(),
            tag: Regex::new(r"<[^>]+>").unwrap(),
            link: Regex::new(r"(https?:&#x2F;&#x2F;[^\s<>()]+)").unwrap(),
        },
        metadata: Metadata {
            instance_name: instance_name,
            instance_description: instance_description,
        },
        config: Config {
            max_note_chars: max_note_chars,
            token_ttl_days: token_ttl_days,
        },
        #[cfg(feature = "web")]
        web_config: WebConfig {
            allow_signup: allow_signup,
            max_sessions_per_user: max_sessions_per_user,
            max_timeline_items: max_timeline_items,
            timezone: timezone,
        },
    }
}
