use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use serde_json::Value;

pub async fn search(state: &AppState, q: &str) -> Result<String, String> {
    if q.starts_with("@") {
        // Search user
        let username = q.trim_start_matches("@");
        if let Some(_existing) = queries::user::get_by_username(state, username).await {
            return Ok(format!("/@{}", username));
        } else {
            // Fetch remote user
            let Some(ap_url) = resolve_acct(state, username).await else {
                return Err("User not found".to_string());
            };
            if let Err(e) = user::add_remote(state, &ap_url).await {
                return Err(format!("Failed to add remote user: {}", e));
            } else {
                return Ok(format!("/@{}", username));
            }
        }
    } else {
        // Search note
        match note::add_remote(state, q).await {
            Ok(note_id) => {
                let note = queries::note::get_by_id(state, note_id).await.unwrap();
                let author = queries::user::get_by_id(state, note.author_id).await;
                return Ok(format!("/@{}/{}", author.username, note.id));
            }
            Err(e) => {
                return Err(format!("Failed to fetch remote note: {}", e));
            }
        }
    }
}

async fn resolve_acct(state: &AppState, acct: &str) -> Option<String> {
    let parts: Vec<&str> = acct.split("@").collect();
    if parts.len() != 2 {
        return None;
    }

    let domain = parts[1];

    let url = format!(
        "https://{}/.well-known/webfinger?resource=acct:{}",
        domain, acct
    );
    let resp = utils::signed_get(state, &url).await;
    if resp.is_err() {
        return None;
    }
    let resp = resp.unwrap();
    let resp_json: Value = resp.json().await.ok()?;

    let Some(links) = resp_json["links"].as_array() else {
        return None;
    };
    for link in links {
        if link["rel"] == "self" {
            return link["href"].as_str().map(|s| s.to_string());
        }
    }

    None
}
