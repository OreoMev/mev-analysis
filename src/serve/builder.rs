use anyhow::Result;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use super::{env::APP_CONFIG, internal_error, ApiResponse, AppState};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Builder {
    extra_data: Option<String>,
    builder_id: String,
    block_count: i64,
}

#[derive(Serialize)]
pub struct BuildersBody {
    builders: Vec<Builder>,
}

pub struct PubkeyBlockCount {
    pub pubkey: String,
    pub block_count: i64,
}

async fn fetch_pubkey_block_counts(relay_pool: &PgPool) -> Result<Vec<PubkeyBlockCount>> {
    let query = format!(
        "
        SELECT
            builder_pubkey AS pubkey,
            COUNT(*) AS block_count
        FROM
            {}_payload_delivered
        GROUP BY
            builder_pubkey
        ",
        &APP_CONFIG.network.to_string()
    );

    sqlx::query(&query)
        .fetch_all(relay_pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| PubkeyBlockCount {
                    pubkey: row.get("pubkey"),
                    block_count: row.get("block_count"),
                })
                .collect()
        })
        .map_err(Into::into)
}

struct BuilderIdMapping {
    builder_id: String,
    pubkey: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuilderBlockCount {
    builder_id: String,
    block_count: i64,
}

async fn get_top_builders_new(relay_pool: &PgPool, mev_pool: &PgPool) -> Result<Vec<Builder>> {
    let counts = fetch_pubkey_block_counts(relay_pool).await?;
    let ids = sqlx::query_as!(
        BuilderIdMapping,
        r#"
        SELECT
            pubkey,
            builder_id
        FROM
            builder_pubkeys
        WHERE
            pubkey = ANY($1)
        "#,
        &counts
            .iter()
            .map(|count| count.pubkey.clone())
            .collect::<Vec<String>>()
    )
    .fetch_all(mev_pool)
    .await?;

    let aggregated = ids
        .iter()
        .fold(HashMap::new(), |mut acc, id| {
            let count = counts
                .iter()
                .find(|count| count.pubkey == id.pubkey)
                .map(|count| count.block_count)
                .unwrap_or(0);

            let entry = acc.entry(id.builder_id.clone()).or_insert(0);
            *entry += count;
            acc
        })
        .into_iter()
        .map(|(builder_id, block_count)| Builder {
            builder_id,
            block_count,
            extra_data: Some("".to_string()),
        })
        .collect();

    Ok(aggregated)
}

pub async fn top_builders(State(state): State<AppState>) -> ApiResponse<BuildersBody> {
    get_top_builders_new(&state.relay_db_pool, &state.mev_db_pool)
        .await
        .map(|builders| Json(BuildersBody { builders }))
        .map_err(internal_error)
}
