use rand::distributions::{Alphanumeric, Distribution};
use rand::rngs::OsRng;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

pub fn date_now() -> String {
    let now = OffsetDateTime::now_utc();
    now.format(&Rfc3339).unwrap()
}

pub fn date_plus_days(days: i64) -> String {
    let date = OffsetDateTime::now_utc() + Duration::days(days);
    date.format(&Rfc3339).unwrap()
}

pub fn gen_secure_token() -> String {
    (0..64)
        .map(|_| Alphanumeric.sample(&mut OsRng) as char)
        .collect()
}
