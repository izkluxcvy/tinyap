use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;
use crate::back::user;

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
        let _ = user::add_remote(state, &author_ap_url).await;
    }

    // Fetch parent notes recursively
    if let Some(in_reply_to) = &in_reply_to {
        let _ = note::add_remote(state, &in_reply_to).await;
    }

    let parent_author_username = if let Some(in_reply_to) = &in_reply_to {
        let parent = queries::note::get_by_ap_url(state, in_reply_to)
            .await
            .unwrap();
        let parent_author = queries::user::get_by_id(state, parent.author_id).await;
        Some(parent_author.username)
    } else {
        None
    };

    // Create note
    let author = queries::user::get_by_ap_url(state, &author_ap_url)
        .await
        .unwrap();

    let _ = note::add(
        state,
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
}
