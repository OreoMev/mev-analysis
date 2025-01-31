use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use sqlx::Row;

use super::{internal_error, AppState, APP_CONFIG};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    inserted_at: DateTime<Utc>,
    block_number: i64,
    value: f64,
}

#[derive(Serialize)]
pub struct PayloadsBody {
    payloads: Vec<Payload>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayloadStatsBody {
    count: i64,
    total_value: f64,
    first_payload_at: DateTime<Utc>,
}

pub async fn delivered_payloads(
    State(state): State<AppState>,
) -> Result<Json<PayloadsBody>, (StatusCode, String)> {
    let query = format!(
        "
        select inserted_at, block_number, (value / 10^18) as value
        from {}_payload_delivered
        order by inserted_at desc
        limit 30
        ",
        &APP_CONFIG.network.to_string()
    );

    sqlx::query(&query)
        .fetch_all(&state.db_pool)
        .await
        .map(|rows| {
            let payloads = rows
                .iter()
                .map(|row| Payload {
                    inserted_at: Utc.from_utc_datetime(&row.get("inserted_at")),
                    block_number: row.get("block_number"),
                    value: row.get("value"),
                })
                .collect();

            Json(PayloadsBody { payloads })
        })
        .map_err(internal_error)
}

pub async fn payload_stats(
    State(state): State<AppState>,
) -> Result<Json<PayloadStatsBody>, (StatusCode, String)> {
    let query = format!(
        "
         select count(*) as count,
                sum(value) / 10^18 as value,
                (select min(inserted_at) from {}_payload_delivered) as first_payload_at
         from {}_payload_delivered
        ",
        &APP_CONFIG.network.to_string(),
        &APP_CONFIG.network.to_string()
    );

    sqlx::query(&query)
        .fetch_one(&state.db_pool)
        .await
        .map(|row| {
            Json(PayloadStatsBody {
                count: row.get("count"),
                total_value: row.get("value"),
                first_payload_at: Utc.from_utc_datetime(&row.get("first_payload_at")),
            })
        })
        .map_err(internal_error)
}

pub async fn top_payloads(
    State(state): State<AppState>,
) -> Result<Json<PayloadsBody>, (StatusCode, String)> {
    let query = format!(
        "
        select inserted_at,
                block_number,
                (value / 10^18) as value
        from {}_payload_delivered
        order by value desc
        limit 10
        ",
        &APP_CONFIG.network.to_string()
    );

    sqlx::query(&query)
        .fetch_all(&state.db_pool)
        .await
        .map(|rows| {
            let payloads = rows
                .iter()
                .map(|row| Payload {
                    inserted_at: Utc.from_utc_datetime(&row.get("inserted_at")),
                    block_number: row.get("block_number"),
                    value: row.get("value"),
                })
                .collect();

            Json(PayloadsBody { payloads })
        })
        .map_err(internal_error)
}
