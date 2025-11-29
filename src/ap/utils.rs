use crate::state::AppState;

use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use reqwest::Client;
use rsa::{
    RsaPrivateKey,
    pkcs1::DecodeRsaPrivateKey,
    pkcs1v15::SigningKey,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description};
use tokio::task;
use url::Url;

pub async fn deliver_signed(
    inbox_url: &str,
    body: &str,
    private_key_pem: &str,
    actor_id: &str,
) -> Result<(), reqwest::Error> {
    println!("Delivering to {}: {}", inbox_url, body);
    let client = Client::new();

    let httpdate_format = format_description::parse(
        "[weekday repr:short], [day] [month repr:short] [year] [hour repr:24]:[minute]:[second] GMT"
    ).unwrap();
    let date = OffsetDateTime::now_utc().format(&httpdate_format).unwrap();

    let url = Url::parse(inbox_url).expect("Invalid inbox URL");
    let host = url.host_str().expect("Inbox URL has no host");
    let path_and_query = {
        let full = url.path().to_owned();
        match url.query() {
            Some(q) => format!("{}?{}", full, q),
            None => full,
        }
    };

    let digest_value = {
        let mut hasher = Sha256::new();
        hasher.update(body.to_string().as_bytes());
        let hash = hasher.finalize();
        format!("SHA-256={}", general_purpose::STANDARD.encode(hash))
    };

    let signing_string = format!(
        "(request-target): post {}\nhost: {}\ndate: {}\ndigest: {}",
        path_and_query, host, date, digest_value
    );

    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem).expect("Invalid private key");
    let signing_key = SigningKey::<Sha256>::new(private_key);

    let signature = signing_key.sign(signing_string.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

    let key_id = format!("{}#main-key", actor_id);

    let signature_header = format!(
        r#"keyId="{}",algorithm="rsa-sha256",headers="(request-target) host date digest",signature="{}""#,
        key_id, signature_b64
    );

    let inbox_url = inbox_url.to_string().clone();
    let date = date.clone();
    let digest_value = digest_value.clone();
    let signature_header = signature_header.clone();
    let body = body.to_string().clone();
    task::spawn(async move {
        let _ = client
            .post(&inbox_url)
            .header("Date", date)
            .header("Digest", digest_value)
            .header("Signature", signature_header)
            .header("Content-Type", "application/activity+json")
            .body(body.to_string())
            .send()
            .await;

        // let req = client
        //     .post(inbox_url)
        //     .header("Date", date)
        //     .header("Digest", digest_value)
        //     .header("Signature", signature_header)
        //     .header("Content-Type", "application/activity+json")
        //     .body(body.to_string())
        //     .build()
        //     .unwrap();
        // debug_post(&client, inbox_url, req).await;
    });

    Ok(())
}

pub async fn fetch_inbox(actor: &str) -> Option<String> {
    let client = Client::new();

    let res = client
        .get(actor)
        .header("Accept", "application/activity+json, application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = res.json().await.ok()?;
    json["inbox"].as_str().map(|s| s.to_string())
}

pub async fn add_notification(
    username: &str,
    notif_type: &str,
    actor: &str,
    note_uuid: Option<&str>,
    state: &AppState,
) {
    let now = OffsetDateTime::now_utc()
        .format(&format_description::well_known::Rfc3339)
        .unwrap();

    sqlx::query!(
        "INSERT INTO notifications (username, type, actor, note_uuid, created_at)
        VALUES (?, ?, ?, ?, ?)",
        username,
        notif_type,
        actor,
        note_uuid,
        now
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub fn strip_html_tags(html: &str) -> String {
    let br_re = Regex::new(r"(?i)<br\s*/?>").unwrap();
    let html = br_re.replace_all(html, "\n");

    let tag_re = Regex::new(r"<[^>]+>").unwrap();

    let text = tag_re.replace_all(&html, "");

    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

// async fn debug_post(client: &Client, _url: &str, req: reqwest::Request) {
//     println!("--- HTTP REQUEST ---");
//     println!("> {} {}", req.method(), req.url());
//     for (k, v) in req.headers() {
//         println!("> {}: {}", k.as_str(), v.to_str().unwrap_or("<invalid>"));
//     }
//     if let Some(body) = req.body() {
//         if let Some(bytes) = body.as_bytes() {
//             println!("> body: {}", String::from_utf8_lossy(bytes));
//         }
//     }

//     let response = client.execute(req).await.unwrap();

//     println!("--- HTTP RESPONSE ---");
//     println!("< STATUS: {}", response.status());
//     for (k, v) in response.headers() {
//         println!("< {}: {}", k.as_str(), v.to_str().unwrap_or("<invalid>"));
//     }
//     let text = response.text().await.unwrap();
//     println!("< body: {}", text);
// }
