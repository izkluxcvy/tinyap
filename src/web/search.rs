use crate::back::init::AppState;
use crate::back::search;
use crate::back::utils::extract_until_id;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub until: Option<i64>,
}

pub async fn get(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Response {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("max_notes", &state.web_config.max_timeline_items);

    let q = query.q.unwrap_or_default();
    let q = q.trim();
    if q.is_empty() {
        return render(&state, &context);
    }
    context.insert("q", q);

    // Get notes
    let (until_date, until_id) = extract_until_id(&state, query.until).await;
    let mut notes = Vec::new();
    match search::search(
        &state,
        q,
        &until_date,
        until_id,
        state.web_config.max_timeline_items,
    )
    .await
    {
        Ok(result) => {
            if let Some(user) = result.user {
                return Redirect::to(&format!("/@{}", user.username)).into_response();
            }
            if let Some(note) = result.note {
                return Redirect::to(&format!("/@{}/{}", note.username, note.id)).into_response();
            }
            notes = result.notes;
        }
        Err(e) => context.insert("error", &e),
    }

    let until_next = if let Some(last_note) = notes.last() {
        last_note.id
    } else {
        until_id
    };

    context.insert("notes", &notes);
    context.insert("until_next", &until_next);

    render(&state, &context)
}

fn render(state: &AppState, context: &tera::Context) -> Response {
    let rendered = state.tera.render("search.html", context).unwrap();

    Html(rendered).into_response()
}
