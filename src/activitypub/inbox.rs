mod accept;
mod announce;
mod create;
mod delete;
mod follow;
mod like;
mod undo;

use crate::back::init::AppState;
use crate::back::utils;

use axum::{
    Json,
    extract::{OriginalUri, State},
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
};
use base64::{Engine as _, engine::general_purpose};
use rsa::{
    RsaPublicKey,
    pkcs1v15::{Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
    signature::Verifier,
};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;

pub async fn post(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    Json(activity): Json<Value>,
) -> impl IntoResponse {
    // Verify signature
    if let Err(e) = verify_signature(&state, &uri, &headers).await {
        return (StatusCode::UNAUTHORIZED, e).into_response();
    }

    println!("Received activity:\n{}", activity);

    // Check required fields
    let Some(actor) = activity["actor"].as_str() else {
        return (StatusCode::BAD_REQUEST, "missing actor").into_response();
    };

    let Some(activity_type) = activity["type"].as_str() else {
        return (StatusCode::BAD_REQUEST, "missing type").into_response();
    };

    // Prevent loopback activity
    if actor.contains("localhost")
        || actor.contains("127.0.0.1")
        || actor.contains("[::1]")
        || actor.contains(&state.domain)
    {
        return (StatusCode::BAD_REQUEST, "loopback not allowed").into_response();
    }

    match activity_type {
        "Follow" => follow::follow(&state, &activity).await,
        "Accept" => accept::follow(&state, &activity).await,
        "Like" => like::like(&state, &activity).await,
        "Announce" => announce::announce(&state, &activity).await,
        "Undo" => {
            let Some(undo_type) = activity["object"]["type"].as_str() else {
                return (StatusCode::BAD_REQUEST, "missing undo type").into_response();
            };
            match undo_type {
                "Follow" => undo::follow(&state, &activity).await,
                "Like" => undo::like(&state, &activity).await,
                "Announce" => undo::announce(&state, &activity).await,
                _ => {}
            }
        }
        "Create" => create::note(&state, &activity).await,
        "Delete" => delete::note(&state, &activity).await,
        _ => {}
    }

    (StatusCode::OK, "activity received").into_response()
}

async fn verify_signature(state: &AppState, uri: &Uri, headers: &HeaderMap) -> Result<(), String> {
    // Get Signature header
    let Some(sig_header) = headers.get("Signature") else {
        return Err("missing Signature header".to_string());
    };
    let Ok(sig_header) = sig_header.to_str() else {
        return Err("invalid signature".to_string());
    };

    // Parse signature header
    let mut sig_map = HashMap::new();
    for part in sig_header.split(',') {
        let Some((k, v)) = part.split_once('=') else {
            return Err("invalid signature".to_string());
        };
        let k = k.trim().to_string();
        let v = v.trim().trim_matches('"').to_string();
        sig_map.insert(k, v);
    }

    // Extract required fields
    let Some(key_id) = sig_map.get("keyId") else {
        return Err("missing keyId".to_string());
    };

    let Some(signed_headers) = sig_map.get("headers") else {
        return Err("missing headers".to_string());
    };
    let signed_headers = signed_headers
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let Some(signature_b64) = sig_map.get("signature") else {
        return Err("missing signature".to_string());
    };
    let Ok(signature) = general_purpose::STANDARD.decode(signature_b64) else {
        return Err("invalid signature".to_string());
    };
    let Ok(signature) = Signature::try_from(signature.as_slice()) else {
        return Err("invalid signature".to_string());
    };

    // Build signing string
    let Some(path_and_query) = uri.path_and_query().map(|pq| pq.as_str()) else {
        return Err("invalid request URI".to_string());
    };
    let mut signing_lines = Vec::new();
    for header in signed_headers {
        if header == "(request-target)" {
            signing_lines.push(format!("(request-target): post {}", path_and_query));
        } else {
            let Some(value) = headers.get(&header) else {
                return Err(format!("missing signed header: {}", header));
            };
            let Ok(value) = value.to_str() else {
                return Err(format!("invalid header value: {}", header));
            };
            signing_lines.push(format!("{}: {}", header.to_lowercase(), value));
        }
    }
    let signing_string = signing_lines.join("\n");

    // Fetch public key
    let Ok(res) = utils::signed_get(state, key_id).await else {
        return Err("failed to fetch public key".to_string());
    };
    let Ok(res_json) = res.json::<Value>().await else {
        return Err("invalid public key response".to_string());
    };
    let Some(public_key_pem) = res_json["publicKey"]["publicKeyPem"].as_str() else {
        return Err("missing public key".to_string());
    };

    // Verify
    let Ok(public_key) = RsaPublicKey::from_public_key_pem(public_key_pem) else {
        return Err("invalid public key".to_string());
    };
    let verifying_key = VerifyingKey::<Sha256>::new(public_key);

    match verifying_key.verify(signing_string.as_bytes(), &signature) {
        Ok(_) => Ok(()),
        Err(_) => Err("signature verification failed".to_string()),
    }
}
