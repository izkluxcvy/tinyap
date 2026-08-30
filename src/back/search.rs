use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;
use crate::back::queries::note::NoteWithAuthorRecord;
use crate::back::queries::user::UserRecord;
use crate::back::user;
use crate::back::utils;

use serde_json::Value;

#[derive(Default)]
pub struct SearchResult {
    pub user: Option<UserRecord>,
    pub note: Option<NoteWithAuthorRecord>,
    pub notes: Vec<NoteWithAuthorRecord>,
}

pub async fn search(
    state: &AppState,
    q: &str,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Result<SearchResult, String> {
    let q = q.trim();

    if let Some(username) = q.strip_prefix("@") {
        // Search user
        let user = match queries::user::get_by_username(state, username).await {
            Some(user) => user,
            None => {
                // Fetch remote user
                let Some(ap_url) = resolve_acct(state, username).await else {
                    return Err("User not found".to_string());
                };
                if let Err(e) = user::add_remote(state, &ap_url).await {
                    return Err(format!("Failed to add remote user: {}", e));
                }
                let Some(user) = queries::user::get_by_username(state, username).await else {
                    return Err("User not found".to_string());
                };
                user
            }
        };

        Ok(SearchResult {
            user: Some(user),
            ..Default::default()
        })
    } else if q.starts_with("http://") || q.starts_with("https://") {
        // Search note
        let note_id = match note::add_remote(state, q, 0).await {
            Ok(note_id) => note_id,
            Err(e) => return Err(format!("Failed to fetch remote note: {}", e)),
        };
        let Some(note) = queries::note::get_with_author_by_id(state, note_id).await else {
            return Err("Note not found".to_string());
        };

        Ok(SearchResult {
            note: Some(note),
            ..Default::default()
        })
    } else {
        // Full text search
        Ok(SearchResult {
            notes: search_notes(state, q, until_date, until_id, limit).await,
            ..Default::default()
        })
    }
}

async fn search_notes(
    state: &AppState,
    q: &str,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    if q.is_empty() {
        return Vec::new();
    }

    match queries::timeline::get_search(state, &to_match_query(q), until_date, until_id, limit)
        .await
    {
        Ok(notes) => notes,
        Err(e) => {
            eprintln!("Failed to search notes: {}", e);
            Vec::new()
        }
    }
}

#[cfg(feature = "sqlite")]
fn to_match_query(q: &str) -> String {
    q.split_whitespace()
        .map(|term| format!("\"{}\"", term.replace("\"", "\"\"")))
        .collect::<Vec<String>>()
        .join(" AND ")
}

#[cfg(feature = "postgres")]
fn to_match_query(q: &str) -> String {
    // websearch_to_tsquery() accepts any input
    q.to_string()
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

    let links = resp_json["links"].as_array()?;
    for link in links {
        if link["rel"] == "self" {
            return link["href"].as_str().map(|s| s.to_string());
        }
    }

    None
}
