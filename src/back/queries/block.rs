use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct BlockRecord {
    pub domain: String,
}

pub async fn get(state: &AppState, domain: &str) -> Option<BlockRecord> {
    query_as(
        "SELECT domain FROM blocks
        WHERE domain = $1",
    )
    .bind(domain)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_list(state: &AppState) -> Vec<BlockRecord> {
    query_as("SELECT domain FROM blocks")
        .fetch_all(&state.db_pool)
        .await
        .unwrap()
}

pub async fn create(state: &AppState, domain: &str) {
    query(
        "INSERT INTO blocks (domain)
        VALUES ($1)",
    )
    .bind(domain)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, domain: &str) {
    query(
        "DELETE FROM blocks
        WHERE domain = $1",
    )
    .bind(domain)
    .execute(&state.db_pool)
    .await
    .unwrap();
}
