use rand::distributions::{Alphanumeric, Distribution};
use rand::rngs::OsRng;
use rand::{Rng, thread_rng};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

pub fn date_now() -> String {
    let now = OffsetDateTime::now_utc();
    now.format(&Rfc3339).unwrap()
}

pub fn date_plus_days(days: i64) -> String {
    let date = OffsetDateTime::now_utc() + Duration::days(days);
    date.format(&Rfc3339).unwrap()
}

// Generate a secure token with 64 random alphanumeric characters
pub fn gen_secure_token() -> String {
    (0..64)
        .map(|_| Alphanumeric.sample(&mut OsRng) as char)
        .collect()
}

/*
*  Generate a unique 64-bit ID with timestamp and random bits
*
*  [                       64 bits ID                       ]
*  [ 0 |      44 bits timestamp      |    19 bits random    ]
*
*  No worries until year 2500
*/
const EPOCH: i64 = 1735689600000; // 2025-01-01
const RANDOM_BITS: i8 = 19;
pub fn gen_unique_id() -> i64 {
    let now_ms = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
    let timestamp = now_ms - EPOCH;

    let random = thread_rng().gen_range(0..(1 << RANDOM_BITS));

    (timestamp << RANDOM_BITS) | random
}

pub fn parse_content(content: &str) -> String {
    let escaped = tera::escape_html(content);
    escaped
}

pub fn user_url(domain: &str, username: &str) -> String {
    format!("https://{}/@{}", domain, username)
}

pub fn local_user_ap_url(domain: &str, username: &str) -> String {
    format!("https://{}/users/{}", domain, username)
}

pub fn local_user_inbox_url(domain: &str, username: &str) -> String {
    format!("https://{}/users/{}/inbox", domain, username)
}

pub fn local_user_outbox_url(domain: &str, username: &str) -> String {
    format!("https://{}/users/{}/outbox", domain, username)
}

pub fn local_note_ap_url(domain: &str, note_id: i64) -> String {
    format!("https://{}/notes/{}", domain, note_id)
}
