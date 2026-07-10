---
type: lld
status: draft
owner: pinakin
summary: "LLD for Typesense: collection schemas, relevance, query DSL compilation, index maintenance, and blue/green rebuilds."
tags: [area/search, release/mvp]
related:
  - ../adr/0006-typesense-search.md
  - storage.md
  - query-api.md
---

# Search

Typesense serves F1 (instant search) and powers command-palette completion ([ADR-0006](../adr/0006-typesense-search.md)). It is a **derived index**: every collection rebuilds from Postgres alone, and the API degrades to Postgres trigram search when Typesense is down. The TUI never talks to Typesense — server-side only.

## Collections

Schemas are code in `owt-search` (shown abridged). Every collection name is versioned (`markets_v3`) behind a stable **alias** (`markets`).

### `markets`

| Field | Type | Facet | Sort | Notes |
|---|---|---|---|---|
| `id` (=market_id) | string | | | document id |
| `slug`, `title`, `description` | string | | | `query_by`, weighted 10/8/2 |
| `outcome_names` | string[] | | | query_by weight 4 |
| `tags` | string[] | ✓ | | |
| `event_slug`, `event_title` | string | | | event_title query_by 5 |
| `status` | string | ✓ | | active/closed/archived |
| `enable_order_book` | bool | ✓ | | |
| `liquidity_score` | float | | ✓ | ranking signal |
| `volume_24h` | float | | ✓ | ranking signal |
| `last_price` | float | | | display |
| `updated_at` | int64 | | ✓ | recency |

`default_sorting_field: liquidity_score`.

### `events`

`id, slug, title, description, tags(facet), status(facet), market_count, top_liquidity(sort), updated_at(sort)`.

### `news`

`id (=news_id), headline, summary, publisher(facet), entities[](facet), event_links[], market_links[], published_at(int64, sort), cluster_id`. `query_by`: headline 10, summary 4, publisher 2.

### `entities`

`id, name, aliases[], description, kind(facet), linked_market_count(sort), news_count_7d(sort)` — powers completion (`/open fed…`, `/topic fed…`) and topic lookup ([topic-lookup](topic-lookup.md)). `query_by`: name 10, aliases 8, description 2; exact/normalized alias matches boost above name-prefix hits so "fed" completes to *Federal Reserve*; `news_count_7d` is the activity tiebreaker.

### `comments` *(v1)*

`id, market_id, body, author_handle, ts(sort)`.

## Relevance

- **Federated query** (`/v1/search`): one multi-search request across collections; results interleaved by normalized `text_match`, with per-type caps (markets 10, events 5, news 10, entities 5) before the API merges.
- **Ranking blend** (markets): `text_match` (primary) → `liquidity_score` (secondary) → `updated_at` (tertiary). Boosts, not filters — a dead-but-exact-match market still surfaces.
- **Typo tolerance:** default 2 typos, min-word-length 4; disabled for slug/token fields.
- **Synonyms:** the entity alias dictionary exports one-way synonyms (`fomc → federal reserve`) on deploy of dictionary changes — search inherits entity resolution's vocabulary without reindexing documents.
- **Relevance regression suite:** a golden-query file (`fixtures/search/golden-queries.toml`: query → expected top-3 ids on the seed dataset) runs in CI; ranking changes must update goldens deliberately.

## Query DSL compilation

The palette DSL ([tui-client](tui-client.md)) compiles server-side to Typesense parameters:

| DSL | Typesense |
|---|---|
| free text | `q`, `query_by` per collection |
| `tag:politics` | `filter_by: tags:=politics` |
| `status:active` | `filter_by: status:=active` |
| `liquidity:>10000` | `filter_by: liquidity_score:>10000` |
| `volume:>1m` | suffix-expanded (`k/m`) numeric filter |
| `is:orderbook` | `filter_by: enable_order_book:=true` |
| `sort:volume` | `sort_by: volume_24h:desc` |
| `type:market,news` | restrict federated set |

Unknown keys are errors (surfaced as palette hints), not silently dropped — the parser lives in `owt-domain` and is shared by search compilation and (v1) alert predicates.

## Indexer

- Durable JetStream consumer `search-indexer` filtered to `canon.v1.market.*.state`, `canon.v1.news.item.*`, and reference-change subjects; batches upserts (`action=upsert`) at 200 docs / 500 ms.
- Deletes are tombstones from status changes (archived markets stay searchable-but-deranked; hard deletes only via rebuild).
- **Index-lag SLO:** ≤ 5 s p95 live (measured `canon` publish → doc visible); ≤ 10 min during backfill bursts. Metric: `search_index_lag_seconds`.

## Rebuild (blue/green)

Triggered by schema-version bump, relevance change, corruption, or restore ([storage § backup](storage.md#backup--restore)):

1. `owtd reindex --collection markets` creates `markets_v{N+1}`.
2. Bulk import **from Postgres only** — never from the old collection or the bus (this is a mandated test scenario).
3. Verify doc count vs canonical count (±0.1 %) + golden queries pass against the new collection.
4. Atomic alias swap `markets → markets_v{N+1}`; old collection kept for one release then dropped.
5. Target: full rebuild of 25 k markets + 500 k news ≤ 10 min self-host.

## Degradation

Health-checked at the API: on Typesense failure, `/v1/search` falls back to Postgres `pg_trgm` similarity over titles/headlines with basic filters — reduced ranking, no typo tolerance, `"degraded": true` in the response envelope so the TUI shows a status-bar notice. Palette completion falls back to prefix matching on slugs.

## Keys & config

Server-side admin key via config/secret; a search-only scoped key is minted for the API process. Typesense version pinned in compose ([deployment](../ops/deployment.md)); memory sizing note: ~philosophy "all searchable text fits in RAM" — the seed dataset needs < 1 GB.

## Related

- [../adr/0006-typesense-search.md](../adr/0006-typesense-search.md)
- [storage.md](storage.md)
- [query-api.md](query-api.md)
