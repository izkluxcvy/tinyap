use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::MaybeAuthUser;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};

pub async fn get(
    State(state): State<AppState>,
    Path((username, id)): Path<(String, i64)>,
    user: MaybeAuthUser,
) -> impl IntoResponse {
    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return "Note not found".into_response();
    };

    // Get author
    let Some(author) = queries::user::get_by_username(&state, &username).await else {
        return "Author not found".into_response();
    };

    // Check privacy
    if note.is_public == 0 {
        if let Some(auth_user_id) = user.id {
            let Some(_follow) = queries::follow::get(&state, auth_user_id, author.id).await else {
                return "Private note".into_response();
            };
        } else {
            return "Private note".into_response();
        }
    }

    // Check is_liked, is_boosted
    let is_liked: bool;
    let is_boosted: bool;
    if let Some(auth_user_id) = user.id {
        // Check like
        if let Some(_like) = queries::like::get(&state, auth_user_id, note.id).await {
            is_liked = true;
        } else {
            is_liked = false;
        }

        // Check boost
        if let Some(_boost) = queries::boost::get(&state, auth_user_id, note.id).await {
            is_boosted = true;
        } else {
            is_boosted = false;
        }
    } else {
        is_liked = false;
        is_boosted = false;
    }

    // Get parent
    let parent = if let Some(parent_id) = note.parent_id {
        queries::note::get_with_author_by_id(&state, parent_id).await
    } else {
        None
    };

    // Get replies
    let replies = queries::note::get_replies_by_parent_id(&state, id).await;

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("parent", &parent);
    context.insert("note", &note);
    context.insert("is_liked", &is_liked);
    context.insert("is_boosted", &is_boosted);
    context.insert("replies", &replies);
    let rendered = state.tera.render("note.html", &context).unwrap();

    Html(rendered).into_response()
}
