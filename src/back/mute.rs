use crate::back::init::AppState;
use crate::back::queries;

pub async fn mute(state: &AppState, muter_id: i64, mutee_id: i64) -> Result<(), String> {
    // Prevent self-mute
    if muter_id == mutee_id {
        return Err("Cannot mute yourself".to_string());
    }

    // Check if already muted
    let existing = queries::mute::get(state, muter_id, mutee_id).await;
    if existing.is_some() {
        return Err("Already muted".to_string());
    }

    // Mute
    queries::mute::create(state, muter_id, mutee_id).await;

    Ok(())
}

pub async fn unmute(state: &AppState, muter_id: i64, mutee_id: i64) -> Result<(), String> {
    // Check if muting
    let existing = queries::mute::get(state, muter_id, mutee_id).await;
    if existing.is_none() {
        return Err("Not muting".to_string());
    }

    // Unmute
    queries::mute::delete(state, muter_id, mutee_id).await;

    Ok(())
}
