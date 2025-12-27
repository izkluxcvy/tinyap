use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

pub async fn add(
    state: &AppState,
    author_id: i64,
    content: &str,
    attachments: Option<&str>,
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
        &id,
        &ap_url,
        &author.id,
        &content,
        attachments,
        &created_at,
        &is_public,
    )
    .await;

    // Update updated_at
    queries::user::update_date(state, author.id, &created_at).await;

    Ok(())
}
