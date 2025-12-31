use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use serde_json::{Value, json};

pub async fn add(
    state: &AppState,
    id: i64,
    ap_url: &str,
    author_id: i64,
    content: &str,
    attachments: Option<String>,
    parent_id: Option<i64>,
    parent_author_username: Option<String>,
    created_at: &str,
    is_public: i64,
) -> Result<(), String> {
    // Create note
    let content = utils::parse_content(content);
    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    queries::note::create(
        state,
        id,
        ap_url,
        author_id,
        &content,
        attachments,
        parent_id,
        parent_author_username,
        &created_at,
        is_public,
    )
    .await;

    // Update updated_at
    queries::user::update_date(state, author_id, &created_at).await;

    // Increment note count
    queries::user::increment_note_count(state, author_id).await;

    Ok(())
}

pub async fn deliver_create(state: &AppState, id: i64, author_id: i64) {
    let note = queries::note::get_by_id(state, id).await.unwrap();
    let author = queries::user::get_by_id(state, author_id).await;
    let note_page_url = utils::note_url(&state.domain, &author.username, id);

    let mut note_object = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": note.ap_url,
        "type": "Note",
        "attributedTo": author.ap_url,
        "content": note.content,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": note.created_at,
        "url": note_page_url,
    });
    let parent_inbox_url: Option<String>;
    if let Some(parent_id) = note.parent_id {
        let parent = queries::note::get_by_id(state, parent_id).await.unwrap();
        let parent_author = queries::user::get_by_id(state, parent.author_id).await;
        parent_inbox_url = Some(parent_author.inbox_url.clone());
        note_object["inReplyTo"] = json!(parent.ap_url);
        note_object["tag"] = json!({
            "type": "Mention",
            "href": parent.ap_url,
            "name": parent_author.username,
        });
    } else {
        parent_inbox_url = None;
    }

    let create_id = format!("{}#create-{}", author.ap_url, utils::gen_unique_id());
    let create_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": create_id,
        "type": "Create",
        "actor": author.ap_url,
        "object": note_object,
    });
    let json_body = create_activity.to_string();

    utils::deliver_to_followers(state, author_id, parent_inbox_url, &json_body).await;
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

    // Fetch parent note recursively
    if let Some(in_reply_to) = &in_reply_to {
        let _res = add_remote(state, &in_reply_to).await;
    }

    // Get parent author username
    let parent_id: Option<i64>;
    let parent_author_username: Option<String>;
    if let Some(in_reply_to) = &in_reply_to {
        if let Some(parent) = queries::note::get_by_ap_url(state, &in_reply_to).await {
            parent_id = Some(parent.id);
            let parent_author = queries::user::get_by_id(state, parent.author_id).await;
            parent_author_username = Some(parent_author.username);
        } else {
            parent_id = None;
            parent_author_username = None;
        };
    } else {
        parent_id = None;
        parent_author_username = None;
    };

    // Create
    let note_id = utils::gen_unique_id();
    queries::note::create(
        state,
        note_id,
        &note_ap_url,
        author.id,
        &content,
        attachments,
        parent_id,
        parent_author_username,
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
                attachments
                    .as_mut()
                    .unwrap()
                    .push_str(&utils::parse_content(url));
                attachments.as_mut().unwrap().push_str("\n");
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

    let created_at = utils::date_to_utc(created_at);

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
        created_at,
        is_public,
    ))
}
