use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use serde_json::Value;

pub async fn add(
    state: &AppState,
    author_id: i64,
    boosted_id: Option<i64>,
    boosted_username: Option<&str>,
    content: &str,
    attachments: Option<&str>,
    in_reply_to: Option<&str>,
    parent_author_username: Option<&str>,
    is_public: i64,
) -> Result<(), String> {
    // Get author
    let author = queries::user::get_by_id(state, author_id).await;

    // Create note
    let id = utils::gen_unique_id();
    let ap_url = utils::local_note_ap_url(&state.domain, id);
    let content = utils::parse_content(content);
    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    let created_at = utils::date_now();

    queries::note::create(
        state,
        id,
        &ap_url,
        author.id,
        boosted_id,
        boosted_username,
        &content,
        attachments,
        in_reply_to,
        parent_author_username,
        &created_at,
        is_public,
    )
    .await;

    // Update updated_at
    queries::user::update_date(state, author.id, &created_at).await;

    // Increment note count
    queries::user::increment_note_count(state, author.id).await;

    Ok(())
}

#[async_recursion::async_recursion]
pub async fn add_remote(state: &AppState, ap_url: &str) -> Result<(), String> {
    // Check if already exists
    if let Some(_existing) = queries::note::get_by_ap_url(state, ap_url).await {
        return Ok(());
    }

    // Fetch
    let Ok(res) = utils::signed_get(state, ap_url).await else {
        return Err("Failed to fetch remote note".to_string());
    };

    let Ok(note_json) = res.json::<Value>().await else {
        return Err("Fetched object is not valid JSON".to_string());
    };

    // Parse
    let Ok((note_ap_url, author_ap_url, content, attachments, in_reply_to, created_at, is_public)) =
        parse_from_json(state, &note_json).await
    else {
        return Err(format!("Failed to parse note JSON"));
    };

    // Create author user if not exists
    let author = if let Some(author) = queries::user::get_by_ap_url(state, &author_ap_url).await {
        author
    } else {
        let res = user::add_remote(state, &author_ap_url).await;
        if let Err(e) = res {
            return Err(format!("Failed to add remote author: {}", e));
        }
        queries::user::get_by_ap_url(state, &author_ap_url)
            .await
            .unwrap()
    };

    if let Some(in_reply_to) = &in_reply_to {
        let existing = queries::note::get_by_ap_url(state, &in_reply_to).await;
        if existing.is_none() {
            let res = add_remote(state, &in_reply_to).await;
            if let Err(_) = res {
                return Err("Failed to fetch conversation notes".to_string());
            }
        };
    }

    let parent_author_username = if let Some(in_reply_to) = &in_reply_to {
        let parent = queries::note::get_by_ap_url(state, &in_reply_to)
            .await
            .unwrap();
        let parent_author = queries::user::get_by_id(state, parent.author_id).await;
        Some(parent_author.username)
    } else {
        None
    };

    // Create
    let note_id = utils::gen_unique_id_from_date(&created_at);
    queries::note::create(
        state,
        note_id,
        &note_ap_url,
        author.id,
        None,
        None,
        &content,
        attachments.as_deref(),
        in_reply_to.as_deref(),
        parent_author_username.as_deref(),
        &created_at,
        is_public,
    )
    .await;

    // Increment note count
    queries::user::increment_note_count(state, author.id).await;

    Ok(())
}

pub async fn parse_from_json(
    state: &AppState,
    note_json: &Value,
) -> Result<
    (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
    ),
    String,
> {
    // Check required fields
    if note_json["type"] != "Note" {
        return Err("Fetched object is not a Note".to_string());
    }

    let Some(note_ap_url) = note_json["id"].as_str() else {
        return Err("Note object missing id".to_string());
    };

    let Some(author_ap_url) = note_json["attributedTo"].as_str() else {
        return Err("Note object missing attributedTo".to_string());
    };

    let Some(content) = note_json["content"].as_str() else {
        return Err("Note object missing content".to_string());
    };

    let content = utils::strip_content(state, content);
    let content = utils::parse_content(&content);

    let mut attachments: Option<String> = None;
    if let Some(note_attachments) = note_json["attachment"].as_array() {
        attachments = Some("".to_string());
        for attachment in note_attachments {
            if let Some(url) = attachment["url"].as_str() {
                attachments.as_mut().unwrap().push_str(url);
                attachments.as_mut().unwrap().push_str(",");
            }
        }
    }

    let in_reply_to = if let Some(in_reply_to) = note_json["inReplyTo"].as_str() {
        Some(in_reply_to.to_string())
    } else {
        None
    };

    let Some(created_at) = note_json["published"].as_str() else {
        return Err("Note object missing published date".to_string());
    };

    let is_public = {
        let Some(to_array) = note_json["to"].as_array() else {
            return Err("Note object missing to field".to_string());
        };

        to_array
            .iter()
            .any(|v| v.as_str().unwrap() == "https://www.w3.org/ns/activitystreams#Public")
            as i64
    };

    Ok((
        note_ap_url.to_string(),
        author_ap_url.to_string(),
        content,
        attachments,
        in_reply_to,
        created_at.to_string(),
        is_public,
    ))
}
