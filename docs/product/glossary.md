---
type: product
status: draft
owner: pinakin
summary: "Canonical vocabulary for the Polymarket domain, the owt domain, and infrastructure terms used across the vault."
tags: [area/product, area/data-model]
related:
  - ../architecture/data-model.md
  - ../architecture/data-sources.md
---

# Glossary

The vault's most-linked note. Every other document uses these terms with exactly these meanings; if a doc needs a different meaning, fix the doc or fix this glossary — never diverge silently.

## Polymarket domain

- **Event** — Polymarket's grouping unit: one real-world question cluster (e.g., "Federal Reserve September Meeting") containing one or more markets.
- **Market** — a single tradable question with discrete outcomes (e.g., "Will the Fed cut rates in September?"). Belongs to exactly one event; identified by `market_id` and `market_slug`.
- **Outcome / outcome price** — each market lists outcomes (often Yes/No) with prices in `[0,1]` that read as **implied probabilities**. Gamma serves `outcomes`/`outcomePrices` as stringified JSON arrays.
- **condition_id** — the Conditional Token Framework identifier binding a market to its on-chain condition; **question_id** — the oracle-side question identifier.
- **token_id** — the identifier of one outcome's tradable token (ERC-1155). A binary market has two token IDs. Order books, prices, and fills key on token IDs.
- **CLOB** — Polymarket's central limit order book: hybrid execution with off-chain matching and on-chain settlement. **CLOB V2** went live 2026-04-28 with new exchange contracts and a rewritten backend; V1-signed orders are no longer accepted.
- **pUSD** — the collateral token introduced with CLOB V2 (replacing USDC-denominated fields in older data).
- **Gamma API** — official discovery surface: events, markets, tags, series, search, comments, profiles.
- **Data API** — official account/analytics surface: trades, positions, holders, open interest, activity.
- **CLOB API** — official market-microstructure surface: order book, prices, midpoints, spreads, price history.
- **WS `market` channel** — official WebSocket channel publishing `book` snapshots and `price_change`, `last_trade_price`, `best_bid_ask` updates per token.
- **RTDS** — Polymarket's real-time data socket: comments plus auxiliary crypto/equity price side-streams; requires an application-level `PING` every 5 seconds.
- **Goldsky** — Polymarket's indexing partner; its V2 datasets (`order_filled`, `orders_matched`, `user_balances`, `user_positions`) are the sanctioned source of on-chain history. Legacy public subgraphs are **incorrect post-V2** and prohibited as sources ([ADR-0007](../adr/0007-goldsky-onchain-truth.md)).
- **Polygon** — the chain all Polymarket contracts live on (chain ID 137).
- **Resolution** — the settlement of a market to a winning outcome per its rules/oracle.
- **Slug** — the human-readable stable identifier in URLs (`will-fed-cut-rates-in-september`).

## owt domain

- **Canonical fact** — an append-only normalized record (price tick, trade, fill, news item, comment, timeline item). Never updated in place; the durable product data.
- **Snapshot** — a mutable latest-state record, e.g. the `market_state` row holding current best bid/ask, last trade, and 24h volume.
- **Envelope** — the normalized wrapper every ingested payload is placed in before hitting the bus: `envelope_id` (UUIDv7), `source`, `kind`, `ts`, `received_at`, `entity_keys`, `payload`, `ingest_version`, `schema_version`, `partition_key`. Normative spec: [data-model](../architecture/data-model.md).
- **source_ref** — a provenance pointer string on canonical records, e.g. `gamma:market:12345`, `clob:token:4833…`, `goldsky:order_filled:0x…`.
- **Entity** — a real-world referent (person, org, topic) that markets and news both mention; **entity resolution** links surface forms ("Fed", "FOMC", "Powell") to one `entity_id` via an **alias** dictionary with human-auditable overrides.
- **Timeline item** — one row in an event's merged chronology (price move, trade burst, news item, comment spike), optionally carrying a `numeric_delta` (price before/after) and a correlation **confidence**.
- **Watchlist / saved view** — persisted research state: a set of pinned markets; a named layout+query combination. Both live server-side under a **workspace** (a single `default` workspace until v2 auth).
- **Alert rule / alert event** — a stored predicate over canonical streams (v1); one firing of that predicate, delivered to the TUI inbox.
- **ingest_version** — the semver of the normalizer that produced a canonical record; bumping it triggers **replay**.
- **Backfill** — bulk historical ingestion via paginated REST. **Replay** — re-running normalization from stored raw envelopes on the bus. **Rehydration** — republishing archived raw envelopes from object storage back onto the bus when the replay window has expired.
- **Conflation** — coalescing rapid updates (book ticks) to the latest value before fanout/render, bounding UI update rates to 4–10 Hz.
- **Correlation** — the price/news linkage job that attaches evidence to price moves with a confidence score.
- **Topic lookup** — the flagship flow from any typed topic to its entity's unified page: context, linked news, related markets, and predictions ([prd-topic-lookup](prd-topic-lookup.md)). A topic **is** an entity ([ADR-0011](../adr/0011-topic-lookup-entity-composite.md)).
- **Aggregated odds / EntityOdds** — the derived per-entity odds document: linked markets grouped by event into facets, same-proposition markets merged into a liquidity-weighted consensus with **dispersion** (spread of member probabilities). Never averages across different questions.
- **Signal vs forecast** — a *signal* is a statistical descriptor of the present (momentum, volatility, volume z-score, book imbalance, divergence); a *forecast* is a calibrated, scoreable probability estimate. Both carry `model_id` + `model_version` and a research disclosure; neither is ever LLM-generated ([ADR-0012](../adr/0012-forecast-derived-data-module.md)).
- **Calibration / longshot bias** — the empirical gap between implied probability and realized outcome frequency; prediction markets systematically overprice longshots. `calib-prob.v1` (v2) corrects for it using resolved-market history.
- **model_version** — the semver of a forecast model; a major bump triggers **rebuild** of that model's derived rows (the forecast analog of `ingest_version`/replay).

## Infrastructure

- **JetStream** — NATS's persistence layer; a **stream** stores messages by subject filter, a **durable consumer** tracks a named cursor through it.
- **Subject** — a NATS topic. owt namespaces: `raw.{source}.{channel}[.{key}]`, `canon.v1.…`, `dlq.{stage}.{source}`. Spec: [realtime-bus](../design/realtime-bus.md).
- **DLQ** — dead-letter queue: where messages go after `max_deliver` failed processing attempts; poison messages land in a Postgres **quarantine** table for operator replay.
- **Checkpoint** — the durable cursor of a backfill/ingest job (`ingest_checkpoints` table), committed in the same transaction as the data it covers.
- **Hypertable** — a TimescaleDB time-partitioned table (chunked by time); **continuous aggregate** — an incrementally-maintained materialized rollup (owt's candles).
- **Collection / alias swap** — a Typesense index and its blue/green rebuild mechanism: build `markets_v{N+1}`, atomically repoint the alias.
- **Token bucket** — the per-(source, endpoint-class) rate limiter primitive (`governor` crate).
- **TEA** — The Elm Architecture: single state, message enum, pure update, view — the TUI's pattern ([tui-client](../design/tui-client.md)).
- **cargo-dist** — release tooling that cross-builds and packages the `owt` binary per platform.
- **SLO / p50 / p95 / p99** — service-level objective and latency percentiles; owt's numbers live in [prd-mvp](prd-mvp.md) and [observability](../ops/observability.md).

## Related

- [../architecture/data-model.md](../architecture/data-model.md)
- [../architecture/data-sources.md](../architecture/data-sources.md)
