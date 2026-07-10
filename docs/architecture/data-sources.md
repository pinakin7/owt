---
type: hld
status: draft
owner: pinakin
summary: "Catalog of every upstream source with authority rank, endpoints, rate limits, failure modes, and compliance caveats."
tags: [area/architecture, area/ingestion, source/polymarket, source/goldsky, source/news, source/x]
related:
  - data-model.md
  - ../design/ingestion.md
  - ../adr/0007-goldsky-onchain-truth.md
  - ../product/glossary.md
---

# Data sources

Every upstream owt ingests, ranked by authority. Ingestion mechanics (schedulers, budgets, state machines) live in [ingestion](../design/ingestion.md); this note is the *what and why* of each source.

**Verification legend:** ✅ = verified against live upstream docs on **2026-07-07** · ⚠ = per the July 2026 research report, re-verify before building against it.

## Authority hierarchy

When two sources disagree about the same fact, the higher rank wins (field-level precedence rules: [normalization](../design/normalization.md)).

| Rank | Source class | Authoritative for |
|---|---|---|
| 1 | Polymarket Gamma / CLOB / Data APIs | current events, markets, books, prices, trades, metadata |
| 2 | Polymarket WebSockets + RTDS | near-real-time market and comment events |
| 3 | Official contracts / V2 docs | token semantics, settlement, collateral (pUSD), addresses |
| 4 | Goldsky Polymarket V2 datasets | historical/replayable on-chain facts post-migration |
| 5 | Publisher APIs / RSS / X | external evidence and narrative |
| 6 | Compliant scraping | gap-filling only, last resort |

## The Polymarket V2 reality (all ✅)

Facts that constrain every design in this vault, verified 2026-07-07:

- **CLOB V2 went live 2026-04-28 (~11:00 UTC)**: new Exchange contracts, rewritten backend, ~1 h downtime, all resting orders wiped at cutover.
- **pUSD** is the collateral token: a standard ERC-20 on Polygon backed 1:1 by USDC, enforced on-chain; API traders wrap USDC.e via the Collateral Onramp `wrap()`.
- **V1-signed orders and legacy V1 SDKs are dead** on production; the EIP-712 domain version is "2" with new verifying-contract addresses.
- **Legacy public subgraphs are incomplete/incorrect for post-migration data** — prohibited as sources ([ADR-0007](../adr/0007-goldsky-onchain-truth.md)).
- All contracts live on **Polygon mainnet, chain ID 137**.

## Rate limits (✅ from the official rate-limits page, 2026-07-07)

Throttling is **Cloudflare queueing**: over-limit requests are *delayed*, not 429-rejected — so detection must watch rising latency, not just status codes. owt budgets default to **≤ 50 % of documented limits** and are config values, never constants ([ingestion](../design/ingestion.md)).

| Surface | Documented limit (per 10 s) |
|---|---|
| General REST (all APIs) | 15,000 |
| Gamma general | 4,000 |
| Gamma `/events` | 500 |
| Gamma `/markets` | 300 |
| Gamma `/public-search` | 350 |
| Data API general | 1,000 |
| Data API `/trades` | 200 |
| Data API `/positions` | 150 |
| CLOB general | 9,000 |
| CLOB `/book`, `/price`, `/midpoint` | 1,500 each |
| CLOB `/prices-history` | 1,000 |
| Trading endpoints (v3 concern) | dual burst + sustained limits |

## Sources

### Polymarket Gamma API — discovery (MVP)

- **Use:** canonical market/event universe: events, markets, tags, series, search, comments, profiles. Drives backfill phase 1 and the metadata sweep.
- **Base URL:** `https://gamma-api.polymarket.com` ⚠. Pagination: offset/limit ⚠.
- **Quirks:** `outcomes`/`outcomePrices` arrive as stringified JSON arrays ⚠; `enableOrderBook` marks CLOB-tradable markets ⚠.
- **Failure modes:** queue-throttling under sweep load; schema drift after platform releases.

### Polymarket CLOB API — microstructure (MVP)

- **Use:** order books, prices, midpoints, spreads, tick sizes, and `/prices-history` for backfill.
- **Base URL:** `https://clob.polymarket.com` ⚠. Market-data endpoints are public; trading endpoints (unused until v3) need L2 auth ⚠.
- **Failure modes:** history fan-out is the rate-budget hog — see call math in [ingestion](../design/ingestion.md).

### Polymarket Data API — trades & positions (MVP: trades)

- **Use:** the MVP trade tape (`/trades`), plus holders/open-interest/positions views (v1+ analytics).
- **Base URL:** `https://data-api.polymarket.com` ⚠.
- **Failure modes:** tightest limits of the three REST surfaces (200/10s trades); schedule accordingly.

### Polymarket WebSocket — CLOB channels (MVP: `market`)

- **Endpoint:** `wss://ws-subscriptions-clob.polymarket.com/ws/` ✅ with channels `market` (public), `user` (auth, v2+), `sports` (v1), `rfq` (unused).
- **`market` channel events** ✅: initial `book` snapshot per subscribed token, then `price_change` (carries side/size/price and best bid/ask), `last_trade_price`, `tick_size_change`. The report's separate `best_bid_ask` message type was not confirmed ⚠ — treat best-bid/ask as fields on `price_change` until fixture-verified.
- **Heartbeat** ✅ (corrects the report): client sends `PING` every 10 s (server answers `PONG`); server pings every 5 s and the client must answer within 10 s or be disconnected.
- **Failure modes:** disconnects under subscription churn; gap risk on reconnect → snapshot-plus-delta resync ([ingestion](../design/ingestion.md)).

### Polymarket RTDS — comments & side-streams (v1)

- **Endpoint:** `wss://ws-live-data.polymarket.com` ✅. Topics: `comments`, `crypto_prices`, `crypto_prices_chainlink`, `equity_prices` ✅.
- **Use:** comment ingestion for timelines and (v1) comment-spike alerts; side-streams optional context.
- **Heartbeat:** same verified ping/pong discipline as above.

### Goldsky Polymarket V2 datasets — on-chain truth (v1)

- **Use:** durable execution facts: `order_filled`, `orders_matched`, `user_balances`, `user_positions` ⚠ (dataset names per report), with historical replay.
- **Caveats:** vendor dependency — tier/pricing verification is an open decision ([open-decisions](../governance/open-decisions.md)); the documented fallback is a self-hosted Polygon indexer (below). Deferred to v1: Data API trades carry the MVP tape.

### Direct Polygon event logs — fallback (reserved)

- **Use:** escape hatch if Goldsky becomes unavailable/unaffordable: index the V2 Exchange contracts' logs from a Polygon RPC (chain 137 ✅). A reserved adapter-crate slot, no MVP work.

### RSS feeds (MVP)

- **Use:** the cheapest broad evidence stream; a configured feed registry (publisher, URL, poll interval, license class) polled on jittered schedules.
- **Caveats:** inconsistent metadata; store full text only where the license class permits — default is metadata + excerpt ([security-and-privacy](../ops/security-and-privacy.md)).

### News APIs — GDELT first (MVP), others later

- **MVP default:** GDELT (free, broad, structured) ⚠ for headline/entity/URL streams.
- **Deferred:** NewsAPI, Event Registry — paid tiers; budget is an open decision.

### X API (v2, deferred)

- **Use (future):** recent search + filtered stream for tracked entities; burst detection on timelines.
- **Caveats:** paid access, policy-sensitive redistribution — schema slots (`social_posts`, `social.*` subjects) are reserved but no MVP/v1 work.

### Compliant scraping (last resort, off by default)

- **Rules:** obey RFC 9309 (robots.txt), per-source terms, conservative rate limits, no full-text retention beyond license; every scraper is a named adapter with an owner and a documented justification for why no API/feed exists.

## Adding a source (contributor path)

A new source = one adapter crate implementing the `BackfillSource`/`StreamSource` traits ([ingestion](../design/ingestion.md)), publishing envelopes to `raw.{source}.*` ([realtime-bus](../design/realtime-bus.md)), plus a mapping table in [normalization](../design/normalization.md) and a row here with authority rank and verification status. No core-crate changes required — this seam is the plugin API.

## Related

- [data-model.md](data-model.md)
- [../design/ingestion.md](../design/ingestion.md)
- [../adr/0007-goldsky-onchain-truth.md](../adr/0007-goldsky-onchain-truth.md)
- [../product/glossary.md](../product/glossary.md)
