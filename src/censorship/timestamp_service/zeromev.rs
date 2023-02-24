mod format;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Duration;
use itertools::Itertools;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use std::str::FromStr;
use tracing::{debug, error};

use self::format::{parse_tx_data, TxTuple};

use super::{ExtractorId, ExtractorTimestamp, TaggedTx, TimestampService, Tx};
use crate::censorship::env::APP_CONFIG;

pub struct ZeroMev {
    db_pool: Pool<Postgres>,
}

impl ZeroMev {
    pub async fn new() -> Self {
        let db_pool = PgPoolOptions::new()
            .max_connections(3)
            .acquire_timeout(Duration::seconds(3).to_std().unwrap())
            .connect(&APP_CONFIG.zeromev_connection_str)
            .await
            .expect("can't connect to zeromev database");

        Self { db_pool }
    }
}

pub type BlockNumber = i64;

#[derive(Debug)]
struct BlockExtractorRow {
    block_number: BlockNumber,
    extractor: ExtractorId,
    tx_data: Vec<TxTuple>,
}

fn tag_transactions(mut txs: Vec<Tx>, mut rows: Vec<BlockExtractorRow>) -> Vec<TaggedTx> {
    // we rely on transaction index to associate timestamps from zeromev to transactions
    // on our side, so we need to make sure everything is sorted
    txs.sort_by_key(|tx| (tx.block_number, tx.tx_index));
    rows.sort_by_key(|row| row.block_number);

    let uniq_nrs = txs
        .iter()
        .map(|tx| tx.block_number)
        .unique()
        .collect_vec()
        .len();

    debug!("uniq nrs {}", uniq_nrs);

    let nrs2 = rows
        .iter()
        .map(|r| r.block_number)
        .unique()
        .collect_vec()
        .len();

    debug!("uniq nrs {}", nrs2);

    debug!(
        "txs: {} to {}",
        txs.first().unwrap().block_number,
        txs.last().unwrap().block_number
    );
    debug!(
        "extractors: {} to {}",
        rows.first().unwrap().block_number,
        rows.last().unwrap().block_number
    );

    let txs_by_block: Vec<(BlockNumber, Vec<Tx>)> = txs
        .into_iter()
        .group_by(|tx| tx.block_number)
        .into_iter()
        .map(|(key, group)| (key, group.into_iter().collect()))
        .collect();

    let extractors_by_block: Vec<(BlockNumber, Vec<BlockExtractorRow>)> = rows
        .into_iter()
        .group_by(|row| row.block_number)
        .into_iter()
        .map(|(key, group)| (key, group.into_iter().collect()))
        .collect();

    debug!(
        "txs_by_block: {}, extractors_by_block: {}",
        txs_by_block.len(),
        extractors_by_block.len()
    );

    if txs_by_block.len() != extractors_by_block.len() {
        error!(
            "expected equal number of blocks when tagging transactions, got {} and {}",
            txs_by_block.len(),
            extractors_by_block.len()
        );
        panic!("expected equal number of blocks when tagging transactions");
    }

    assert!(
        txs_by_block.len() == extractors_by_block.len(),
        "expected equal number of blocks when tagging transactions"
    );

    txs_by_block
        .iter()
        .zip(extractors_by_block.iter())
        .map(|((b0, txs), (b1, extractors))| {
            assert!(b0 == b1, "mismatched block numbers during zip");

            let extractor_tx_counts_match = extractors
                .iter()
                .map(|ex| ex.tx_data.len())
                .unique()
                .into_iter()
                .count()
                == 1;

            assert!(
                extractor_tx_counts_match,
                "expected every extractor to contain equal amount of txs"
            );

            txs.iter().map(|tx| {
                let timestamps: Vec<ExtractorTimestamp> = extractors
                    .iter()
                    .map(|ex| ExtractorTimestamp {
                        id: ex.extractor.clone(),
                        timestamp: ex
                            .tx_data
                            .get(usize::try_from(tx.tx_index).unwrap())
                            .expect("expected extractor data to contain transaction index")
                            .0,
                    })
                    .collect();

                TaggedTx {
                    timestamps,
                    tx: tx.clone(),
                }
            })
        })
        .flatten()
        .collect()
}

#[async_trait]
impl TimestampService for ZeroMev {
    async fn fetch_tx_timestamps(&self, txs: Vec<Tx>) -> Result<Vec<TaggedTx>> {
        let start_block = txs
            .first()
            .expect("fetch_tx_timestamps received empty vector")
            .block_number;

        let end_block = txs
            .last()
            .expect("fetch_tx_timestamps received empty vector")
            .block_number;

        // In cases where there are duplicate extractors for a block, use the most recent
        // https://stackoverflow.com/a/45018194
        let query = format!(
            "
            SELECT DISTINCT ON (block_number, extractor)
                block_number,
                block_time,
                extractor.code AS extractor,
                tx_data
            FROM
                extractor_block
            INNER JOIN extractor USING (extractor_index)
            WHERE
                block_number >= {}
                AND block_number <= {}
            ORDER BY
                block_number, extractor, block_time DESC
            ",
            start_block, end_block
        );

        let results: Vec<BlockExtractorRow> = sqlx::query(&query)
            .fetch_all(&self.db_pool)
            .await
            .map(|rows| {
                rows.iter()
                    .map(|row| BlockExtractorRow {
                        block_number: row.get("block_number"),
                        extractor: ExtractorId::from_str(&row.get::<String, _>("extractor"))
                            .expect("failed to parse extractor id"),
                        tx_data: parse_tx_data(row.get("tx_data")),
                    })
                    .collect()
            })?;

        Ok(tag_transactions(txs, results))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::ops::Add;

    use super::{tag_transactions, BlockExtractorRow, ExtractorId, Tx};

    #[test]
    fn test_tag_transactions() {
        let txs = vec![
            Tx {
                tx_hash: "lol123".to_string(),
                tx_index: 0,
                block_number: 1000,
                max_fee: None,
                max_prio_fee: None,
                address_trace: vec![],
            },
            Tx {
                tx_hash: "bal234".to_string(),
                tx_index: 1,
                block_number: 1000,
                max_fee: None,
                max_prio_fee: None,
                address_trace: vec![],
            },
        ];

        let d0 = Utc::now();
        let d1 = d0.add(Duration::seconds(10));

        let rows = vec![
            BlockExtractorRow {
                block_number: 1000,
                extractor: ExtractorId::ZMevUS,
                tx_data: vec![(d0, 1000), (d0, 1000)],
            },
            BlockExtractorRow {
                block_number: 1000,
                extractor: ExtractorId::ZMevEU,
                tx_data: vec![(d1, 1000), (d1, 1000)],
            },
        ];

        let res = tag_transactions(txs, rows);

        assert_eq!(res.len(), 2);

        assert_eq!(res[0].tx.tx_hash, "lol123".to_string());
        assert_eq!(res[0].timestamps[0].id, ExtractorId::ZMevUS);
        assert_eq!(res[0].timestamps[0].timestamp, d0);
        assert_eq!(res[0].timestamps[1].id, ExtractorId::ZMevEU);
        assert_eq!(res[0].timestamps[1].timestamp, d1);

        assert_eq!(res[1].tx.tx_hash, "bal234".to_string());
        assert_eq!(res[1].timestamps[0].id, ExtractorId::ZMevUS);
        assert_eq!(res[1].timestamps[0].timestamp, d0);
        assert_eq!(res[1].timestamps[1].id, ExtractorId::ZMevEU);
        assert_eq!(res[1].timestamps[1].timestamp, d1);
    }
}
