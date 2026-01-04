use crate::VERSION;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use serde_json::json;

pub async fn get_well_known(State(state): State<AppState>) -> impl IntoResponse {
    let mut json_headers = HeaderMap::new();
    json_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    let json_body = json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": format!("https://{}/nodeinfo/2.1", state.domain)
            }
        ]
    });

    (json_headers, Json(json_body)).into_response()
}

pub async fn get_nodeinfo(State(state): State<AppState>) -> impl IntoResponse {
    let total_users = queries::user::count_total(&state).await;
    let active_users = queries::user::count_active(&state, &utils::date_plus_days(-30)).await;
    #[cfg(feature = "web")]
    let open_registrations = state.web_config.allow_signup;
    #[cfg(not(feature = "web"))]
    let open_registrations = false;

    let mut json_headers = HeaderMap::new();
    json_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    let json_body = json!({
        "version": "2.1",
        "software": {
            "name": "tinyap",
            "version": VERSION,
        },
        "protocols": ["activitypub"],
        "services": {
            "inbound": [],
            "outbound": [],
        },
        "openRegistrations": open_registrations,
        "usage": {
            "users": {
                "total": total_users,
                "activeMonth": active_users,
            },
        },
        "metadata": {
            "nodeName": state.metadata.instance_name,
            "nodeDescription": state.metadata.instance_description,
        },
    });

    (json_headers, Json(json_body)).into_response()
}
