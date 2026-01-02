use crate::back::init::AppState;
use crate::back::queries;

use base64::{Engine as _, engine::general_purpose};
use rand::distributions::{Alphanumeric, Distribution};
use rand::rngs::OsRng;
use rand::{Rng, thread_rng};
use rsa::{
    RsaPrivateKey,
    pkcs1::DecodeRsaPrivateKey,
    pkcs1v15::SigningKey,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::task;
use url::Url;

pub fn date_now() -> String {
    let now = OffsetDateTime::now_utc();
    now.format(&Rfc3339).unwrap()
}

pub fn date_plus_days(days: i64) -> String {
    let date = OffsetDateTime::now_utc() + Duration::days(days);
    date.format(&Rfc3339).unwrap()
}

pub fn date_now_http_format() -> String {
    let now = OffsetDateTime::now_utc();
    let format = time::format_description::parse(
        "[weekday repr:short], [day] [month repr:short] [year] [hour repr:24]:[minute]:[second] GMT"
    ).unwrap();
    now.format(&format).unwrap()
}

pub fn date_to_utc(date: &str) -> String {
    let parsed = OffsetDateTime::parse(date, &Rfc3339).unwrap();
    parsed.format(&Rfc3339).unwrap()
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
const EPOCH: i64 = 1767225600000; // 2026-01-01
const RANDOM_BITS: i8 = 19;
pub fn gen_unique_id() -> i64 {
    let now_ms = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
    let timestamp = now_ms - EPOCH;

    let random = thread_rng().gen_range(0..(1 << RANDOM_BITS));

    (timestamp << RANDOM_BITS) | random
}

pub fn strip_content(state: &AppState, content: &str) -> String {
    let content = content.trim();
    let content = state.re.tag.replace_all(content, "");
    let content = content
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let content = if content.chars().count() > state.config.max_note_chars {
        let byte_end = content
            .char_indices()
            .nth(state.config.max_note_chars)
            .unwrap()
            .0;
        content[..byte_end].to_string()
    } else {
        content
    };

    content
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

pub fn note_url(domain: &str, author: &str, id: i64) -> String {
    format!("https://{}/@{}/{}", domain, author, id)
}

pub fn local_note_ap_url(domain: &str, id: i64) -> String {
    format!("https://{}/notes/{}", domain, id)
}

pub async fn signed_deliver(
    state: &AppState,
    sender_ap_url: &str,
    private_key: &str,
    recipient_inbox: &str,
    body: &str,
) {
    println!("Delivering to {}:\n{}", recipient_inbox, body);

    // Sign
    let date = date_now_http_format();

    let digest_value = {
        let mut hasher = Sha256::new();
        hasher.update(body.to_string().as_bytes());
        let hash = hasher.finalize();
        format!("SHA-256={}", general_purpose::STANDARD.encode(hash))
    };

    let url_parsed = Url::parse(recipient_inbox).unwrap();
    let host = url_parsed.host_str().unwrap();
    let path_and_query = {
        let full = url_parsed.path();
        match url_parsed.query() {
            Some(q) => format!("{}?{}", &full, q),
            None => full.to_string(),
        }
    };

    let signing_string = format!(
        "(request-target): post {}\nhost: {}\ndate: {}\ndigest: {}",
        path_and_query, host, date, digest_value
    );

    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key).unwrap();
    let signing_key = SigningKey::<Sha256>::new(private_key);

    let signature = signing_key.sign(signing_string.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

    let key_id = format!("{}#main-key", sender_ap_url);
    let signed_header = format!(
        r#"keyId="{}",algorithm="rsa-sha256",headers="(request-target) host date digest",signature="{}""#,
        key_id, signature_b64
    );

    // Deliver in background queue
    task::spawn({
        let deliver_queue = state.deliver_queue.clone();
        let http_client = state.http_client.clone();
        let inbox = recipient_inbox.to_string().clone();
        let body = body.to_string().clone();
        async move {
            let _permit = deliver_queue.acquire().await.unwrap();

            let _res = http_client
                .post(inbox)
                .header("Date", date)
                .header("Digest", digest_value)
                .header("Signature", signed_header)
                .header("Content-Type", "application/activity+json")
                .body(body.to_string())
                .send()
                .await;

            drop(_permit);

            // let status = _res.as_ref().unwrap().status();
            // let body = _res.unwrap().text().await.unwrap();
            // println!("{}, response: {}", status, body);
        }
    });
}

pub async fn deliver_to_followers(
    state: &AppState,
    sender_id: i64,
    parent_inbox: Option<String>,
    body: &str,
) {
    let sender = queries::user::get_by_id(state, sender_id).await;
    let private_key = sender.private_key.unwrap();

    let followers = queries::follow::get_follower_inboxes(state, sender_id).await;
    let mut already_delivered_hosts = vec![state.domain.clone()];
    for follower in followers {
        let url_parsed = Url::parse(&follower.inbox_url).unwrap();
        let host = url_parsed.host_str().unwrap().to_string();
        if !already_delivered_hosts.contains(&host) {
            signed_deliver(
                state,
                &sender.ap_url,
                &private_key,
                &follower.inbox_url,
                body,
            )
            .await;
            already_delivered_hosts.push(host);
        }
    }

    // Also deliver to parent author if exists
    if let Some(parent_inbox) = parent_inbox {
        let url_parsed = Url::parse(&parent_inbox).unwrap();
        let host = url_parsed.host_str().unwrap().to_string();
        if !already_delivered_hosts.contains(&host) {
            signed_deliver(state, &sender.ap_url, &private_key, &parent_inbox, body).await;
        }
    }
}

pub async fn signed_get(state: &AppState, url: &str) -> Result<reqwest::Response, String> {
    let sender = queries::user::get_temp_sign_user(state).await;
    let sender_ap_url = &sender.ap_url;
    let private_key = sender.private_key.unwrap();
    // Sign
    let date = date_now_http_format();

    let Ok(url_parsed) = Url::parse(url) else {
        return Err("Invalid URL".to_string());
    };
    let host = url_parsed.host_str().unwrap();
    let path_and_query = {
        let full = url_parsed.path();
        match url_parsed.query() {
            Some(q) => format!("{}?{}", &full, q),
            None => full.to_string(),
        }
    };

    let signing_string = format!(
        "(request-target): get {}\nhost: {}\ndate: {}",
        path_and_query, host, date
    );

    let private_key = RsaPrivateKey::from_pkcs1_pem(&private_key).unwrap();
    let signing_key = SigningKey::<Sha256>::new(private_key);

    let signature = signing_key.sign(signing_string.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

    let key_id = format!("{}#main-key", sender_ap_url);
    let signed_header = format!(
        r#"keyId="{}",algorithm="rsa-sha256",headers="(request-target) host date",signature="{}""#,
        key_id, signature_b64
    );

    // Get
    let res = state.http_client
        .get(url)
        .header("Host", host)
        .header("Date", date)
        .header("Signature", signed_header)
        .header("Accept", "application/activity+json, application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
        .send()
        .await;

    match res {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("HTTP GET error: {}", e)),
    }
}
