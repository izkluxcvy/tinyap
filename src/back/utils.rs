use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn date_now() -> String {
    let now = OffsetDateTime::now_utc();
    now.format(&Rfc3339).unwrap()
}
