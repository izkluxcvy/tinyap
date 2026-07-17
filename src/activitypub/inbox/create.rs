use crate::back::init::AppState;
use crate::back::note;
use crate::back::notification;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use serde_json::Value;

pub async fn note(state: &AppState, activity: &Value) {
    let note_object = &activity["object"];

    let Ok((note_ap_url, author_ap_url, content, attachments, in_reply_to, created_at, is_public)) =
        note::parse_from_json(state, note_object).await
    else {
        return;
    };

    // Check if already exists
    let existing = queries::note::get_by_ap_url(state, &note_ap_url).await;
    if existing.is_some() {
        return;
    }

    // Create or update author
    if let Some(author) = queries::user::get_by_ap_url(state, &author_ap_url).await {
        let _ = user::update_remote(state, &author.ap_url).await;
    } else {
        if let Err(e) = user::add_remote(state, &author_ap_url).await {
            println!("Failed to add remote user {}: {}", author_ap_url, e);
            return;
        }
    }

    // Fetch parent notes recursively
    if let Some(in_reply_to) = &in_reply_to {
        let _ = note::add_remote(state, in_reply_to, 0).await;
    }

    let parent_id: Option<i64>;
    let parent_author_username: Option<String>;
    if let Some(in_reply_to) = &in_reply_to {
        if let Some(parent) = queries::note::get_by_ap_url(state, in_reply_to).await {
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

    // Create note
    let id = utils::gen_unique_id();
    let author = queries::user::get_by_ap_url(state, &author_ap_url)
        .await
        .unwrap();

    // Extract mentions
    let mentioned_usernames = if let Some(mentions) = note_object["tag"].as_array() {
        mentions
            .iter()
            .filter_map(|tag| {
                if tag["type"] == "Mention"
                    && tag["name"].as_str().unwrap_or("").contains(&state.domain)
                {
                    tag["name"].as_str().map(|s| {
                        s.trim_start_matches('@')
                            .trim_end_matches(&format!("@{}", state.domain))
                            .to_string()
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
    } else if let Some(mention) = note_object["tag"]["name"].as_str() {
        if mention.contains(&state.domain) {
            vec![
                mention
                    .trim_start_matches('@')
                    .trim_end_matches(&format!("@{}", state.domain))
                    .to_string(),
            ]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let parent_author_uname = parent_author_username.clone().unwrap_or_default();

    // Create note
    let _ = note::add(
        state,
        id,
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

    // Add notifications for mentions
    let mentioned_users = queries::user::get_by_username_in(state, &mentioned_usernames).await;
    for mentioned_user in mentioned_users {
        if mentioned_user.username == parent_author_uname {
            continue;
        }
        println!(
            "user {} mentioned user {} in note {}",
            author.username, mentioned_user.username, note_ap_url
        );
        notification::add(
            state,
            notification::EventType::Mention,
            author.id,
            mentioned_user.id,
            Some(id),
        )
        .await;
    }
}
