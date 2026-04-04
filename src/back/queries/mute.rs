use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct MuteRecord {
    pub _id: i64,
}

pub async fn get(state: &AppState, muter_id: i64, mutee_id: i64) -> Option<MuteRecord> {
    query_as(
        "SELECT id AS _id FROM mutes
        WHERE muter_id = $1
        AND mutee_id = $2",
    )
    .bind(muter_id)
    .bind(mutee_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(state: &AppState, muter_id: i64, mutee_id: i64) {
    query(
        "INSERT INTO mutes (muter_id, mutee_id)
        VALUES ($1, $2)",
    )
    .bind(muter_id)
    .bind(mutee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, muter_id: i64, mutee_id: i64) {
    query(
        "DELETE FROM mutes
        WHERE muter_id = $1
        AND mutee_id = $2",
    )
    .bind(muter_id)
    .bind(mutee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}
