use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

pub enum EventType {
    Follow,
    Reply,
    Like,
    Boost,
}

impl From<EventType> for i64 {
    fn from(value: EventType) -> i64 {
        match value {
            EventType::Follow => 1,
            EventType::Reply => 2,
            EventType::Like => 3,
            EventType::Boost => 4,
        }
    }
}

pub async fn add(
    state: &AppState,
    event_type: EventType,
    sender_id: i64,
    recipient_id: i64,
    note_id: Option<i64>,
) {
    let date_now = utils::date_now();

    // Check if sender and recipient are the same
    if sender_id == recipient_id {
        return;
    }

    // Check if recipient is local
    let recipient = queries::user::get_by_id(state, recipient_id).await;
    if recipient.is_local == 0 {
        return;
    }

    queries::notification::create(
        state,
        event_type.into(),
        sender_id,
        recipient_id,
        note_id,
        &date_now,
    )
    .await;
}
