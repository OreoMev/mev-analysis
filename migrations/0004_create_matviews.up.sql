---
-- Builders
--

CREATE MATERIALIZED VIEW IF NOT EXISTS builders_7d AS
SELECT
   builder_pubkeys.builder_id,
   count(transactions_data.transaction_hash) AS count,
   CASE
      WHEN
         max(bb.censoring) = 1
      THEN
         'yes'::text
      ELSE
         'no'::text
   END
   AS censoring
FROM
   transactions_data
   LEFT JOIN
      block_production
      ON block_production.block_number = transactions_data.block_number
   LEFT JOIN
      builder_pubkeys
      ON builder_pubkeys.pubkey::text = block_production.builder_pubkey::text
   LEFT JOIN
      (
         SELECT DISTINCT
            builder_pubkeys_1.builder_id AS b_id,
            1 AS censoring
         FROM
            transactions_data transactions_data_1
            LEFT JOIN
               block_production block_production_1
               ON block_production_1.block_number < transactions_data_1.block_number
               AND block_production_1.block_number >=
               (
                  transactions_data_1.block_number - transactions_data_1.blocksdelay
               )
            LEFT JOIN
               builder_pubkeys builder_pubkeys_1
               ON builder_pubkeys_1.pubkey::text = block_production_1.builder_pubkey::text
         WHERE
            transactions_data_1.blacklist <> '{NULL}'::text[]
            AND
            (
               transactions_data_1.lowbasefee + transactions_data_1.lowtip + transactions_data_1.congested
            )
            = 0
      )
      bb
      ON bb.b_id = builder_pubkeys.builder_id
WHERE
   transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
GROUP BY
   builder_pubkeys.builder_id
WITH NO DATA;


CREATE MATERIALIZED VIEW IF NOT EXISTS builders_30d AS
SELECT
   builder_pubkeys.builder_id,
   count(transactions_data.transaction_hash) AS count,
   CASE
      WHEN
         max(bb.censoring) = 1
      THEN
         'yes'::text
      ELSE
         'no'::text
   END
   AS censoring
FROM
   transactions_data
   LEFT JOIN
      block_production
      ON block_production.block_number = transactions_data.block_number
   LEFT JOIN
      builder_pubkeys
      ON builder_pubkeys.pubkey::text = block_production.builder_pubkey::text
   LEFT JOIN
      (
         SELECT DISTINCT
            builder_pubkeys_1.builder_id AS b_id,
            1 AS censoring
         FROM
            transactions_data transactions_data_1
            LEFT JOIN
               block_production block_production_1
               ON block_production_1.block_number < transactions_data_1.block_number
               AND block_production_1.block_number >=
               (
                  transactions_data_1.block_number - transactions_data_1.blocksdelay
               )
            LEFT JOIN
               builder_pubkeys builder_pubkeys_1
               ON builder_pubkeys_1.pubkey::text = block_production_1.builder_pubkey::text
         WHERE
            transactions_data_1.blacklist <> '{NULL}'::text[]
            AND
            (
               transactions_data_1.lowbasefee + transactions_data_1.lowtip + transactions_data_1.congested
            )
            = 0
      )
      bb
      ON bb.b_id = builder_pubkeys.builder_id
WHERE
   transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
GROUP BY
   builder_pubkeys.builder_id
WITH NO DATA;



---
-- Censored transactions
--

CREATE MATERIALIZED VIEW IF NOT EXISTS censored_transactions_7d AS
  SELECT
    transactions_data.transaction_hash,
    transactions_data.block_number,
    transactions_data.mined,
    transactions_data.delay,
    transactions_data.blacklist,
    transactions_data.blocksdelay
  FROM
    transactions_data
  WHERE
  (
    transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.congested
  )
  = 0
  AND transactions_data.blocksdelay > 0
  AND transactions_data.blacklist <> '{NULL}'::text[]
  AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
WITH NO DATA;

CREATE MATERIALIZED VIEW IF NOT EXISTS censored_transactions_30d AS
  SELECT
    transactions_data.transaction_hash,
    transactions_data.block_number,
    transactions_data.mined,
    transactions_data.delay,
    transactions_data.blacklist,
    transactions_data.blocksdelay
  FROM
    transactions_data
  WHERE
  (
    transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.congested
  )
  = 0
  AND transactions_data.blocksdelay > 0
  AND transactions_data.blacklist <> '{NULL}'::text[]
  AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
WITH NO DATA;


---
-- Inclusion delay
--

CREATE MATERIALIZED VIEW IF NOT EXISTS inclusion_delay_7d AS
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'ofac'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist <> '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'ofac_delayed'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist <> '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.blocksdelay > 0
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'normal'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'low_base_fee'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.lowbasefee = 1
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'low_tip'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.lowtip = 1
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   0 AS avg_delay,
   0 AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'miner'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip
   )
   = 0
   AND transactions_data.minertransaction = 1
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'congested'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.congested = 1
   AND transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
WITH NO DATA;

CREATE MATERIALIZED VIEW IF NOT EXISTS inclusion_delay_30d AS
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'ofac'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist <> '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'ofac_delayed'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist <> '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.blocksdelay > 0
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'normal'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'low_base_fee'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.lowbasefee = 1
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'low_tip'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.lowtip = 1
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   0 AS avg_delay,
   0 AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'miner'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.congested + transactions_data.lowbasefee + transactions_data.lowtip
   )
   = 0
   AND transactions_data.minertransaction = 1
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
UNION
SELECT
   avg(transactions_data.delay) AS avg_delay,
   avg(transactions_data.blocksdelay) AS avg_block_delay,
   count(transactions_data.transaction_hash) AS n,
   'congested'::text AS t_type
FROM
   transactions_data
WHERE
   transactions_data.blacklist = '{NULL}'::text[]
   AND
   (
      transactions_data.lowbasefee + transactions_data.lowtip + transactions_data.minertransaction
   )
   = 0
   AND transactions_data.congested = 1
   AND transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
WITH NO DATA;



---
-- Operators
--

CREATE MATERIALIZED VIEW IF NOT EXISTS operators_7d AS
  SELECT
    validator_pubkeys.operator_id,
    count(transactions_data.transaction_hash) AS count,
    'yes'::text AS censoring
  FROM
    transactions_data
  LEFT JOIN
    block_production
    ON block_production.block_number = transactions_data.block_number
  LEFT JOIN
    validator_pubkeys
    ON validator_pubkeys.pubkey::text = block_production.proposer_pubkey::text
  WHERE
    transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
  GROUP BY
    validator_pubkeys.operator_id
WITH NO DATA;


CREATE MATERIALIZED VIEW IF NOT EXISTS operators_30d AS
  SELECT
    validator_pubkeys.operator_id,
    count(transactions_data.transaction_hash) AS count,
    'yes'::text AS censoring
  FROM
    transactions_data
  LEFT JOIN
    block_production
    ON block_production.block_number = transactions_data.block_number
  LEFT JOIN
    validator_pubkeys
    ON validator_pubkeys.pubkey::text = block_production.proposer_pubkey::text
  WHERE
    transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
  GROUP BY
    validator_pubkeys.operator_id
WITH NO DATA;



---
-- Top delay
--

CREATE MATERIALIZED VIEW IF NOT EXISTS top_7d AS
  SELECT
    transactions_data.blocksdelay,
    transactions_data.blacklist,
    transactions_data.delay,
    transactions_data.transaction_hash
  FROM
    transactions_data
  WHERE
    transactions_data.mined > (CURRENT_DATE - '7 days'::interval)
    AND transactions_data.blacklist <> '{NULL}'::text[]
  ORDER BY
    transactions_data.delay DESC LIMIT 10
WITH NO DATA;

CREATE MATERIALIZED VIEW IF NOT EXISTS top_30d AS
  SELECT
    transactions_data.blocksdelay,
    transactions_data.blacklist,
    transactions_data.delay,
    transactions_data.transaction_hash
  FROM
    transactions_data
  WHERE
    transactions_data.mined > (CURRENT_DATE - '30 days'::interval)
    AND transactions_data.blacklist <> '{NULL}'::text[]
  ORDER BY
    transactions_data.delay DESC LIMIT 10
WITH NO DATA;
