---
type: adr
status: draft
owner: pinakin
summary: "Topic lookup composes the existing entity model: a composite entities read with event-grouped aggregated odds as derived data; no parallel Topic noun."
tags: [area/api, area/data-model]
related:
  - ../design/topic-lookup.md
  - ../design/query-api.md
  - ../architecture/data-model.md
  - ../product/prd-topic-lookup.md
  - 0012-forecast-derived-data-module.md
---

# ADR-0011: Topic lookup composes the entity model

## Context

The flagship "topic lookup" feature ([prd-topic-lookup](../product/prd-topic-lookup.md)) lets a user type any real-world topic — "Fed rate cut", "Bitcoin", "US election" — and get one unified page: context, linked news, related markets and events, and the predictions that apply to it (aggregated market odds now, model forecasts in v1 per [ADR-0012](0012-forecast-derived-data-module.md)).

The vault already defines the primitive this needs: **Entity** (`ent_{kebab-name}`, kind ∈ person | org | topic | place) with an alias dictionary, `entity_links` to markets and events, entity-tagged news, and a Typesense `entities` collection powering completion ([data-model](../architecture/data-model.md), [normalization](../design/normalization.md), [search](../design/search.md)). Introducing a separate "Topic" noun would duplicate that resolution machinery and collide with the existing entity kind `topic`.

This crosses two mandatory-ADR lines ([adr process](README.md)): the public API contract gains surface, and the canonical schema gains a field.

## Decision

1. **Topic = Entity.** There is no Topic noun, table, or endpoint. Topic lookup is a composite read over the entity model: `GET /v1/entities/{id}?include=markets,events,news:10,odds` — the same explicit `include` expansion the market-detail composite uses ([query-api](../design/query-api.md#composite-screen-loads)). Resolution from free text to `entity_id` is the existing search-first flow over the `entities` collection.
2. **Aggregated odds are derived data, never canonical facts.** Linked markets are grouped **by event**; only same-proposition markets merge into one consensus number (liquidity-weighted); different questions under one topic render as separate facets. Full algorithm: [topic-lookup](../design/topic-lookup.md).
3. **Odds are computed at read time in MVP, materialized in v1.** MVP computes the `EntityOdds` document on request from `entity_links` joined with `market_state` (no new tables); the topic screen goes live via the existing `market:{slug}:state` WS topics. In v1 the forecast module materializes `entity_odds` and pushes WS topic `entity:{id}:odds` ([ADR-0012](0012-forecast-derived-data-module.md)).
4. **`Entity` gains a nullable `description`** — a human-curated context blurb (never generated), additive within the current `schema_version`.
5. **Forecast seams are reserved, not designed here:** `GET /v1/entities/{id}/forecasts`, `GET /v1/markets/{slug}/forecasts`, and WS topics `entity:{id}:odds` / `market:{slug}:forecasts` are v1 surface owned by ADR-0012.

## Consequences

- The topic page ships in MVP from data MVP already ingests (Gamma, CLOB, news, entity dictionary) — no new module, no new tables, no new WS topics.
- Entity quality becomes product-critical: the curated starter dictionary and `description` blurbs are now user-facing content, not just linking metadata ([normalization](../design/normalization.md#entity-resolution-mvp-dictionary--rules)).
- The API stays additive-only under `/v1` ([ADR-0010](0010-rest-ws-api-protocol.md)); the TUI gains one screen and one verb ([tui-client](../design/tui-client.md)).
- Aggregation semantics live server-side in one place; every client renders the same consensus numbers.
- An entity with zero linked markets must render an honest empty state — lookup never fabricates relevance.

## Alternatives considered

- **New `GET /v1/topics/{slug}` noun** — duplicates entity resolution, collides with entity kind `topic`, and forces two dictionaries to stay in sync. Rejected.
- **Materialized odds table at MVP** — an indexed `entity_links ⋈ market_state` join serves the read within budget; a table would add a writer, a rebuild path, and staleness bugs before any WS-push need exists. Deferred to v1 where the forecast module needs it anyway.
- **Client-side aggregation** — the TUI is deliberately thin ([ADR-0002](0002-ratatui-tui-first-client.md)); odds semantics (grouping, weighting, staleness) are product logic the server must own.

## Rollout notes

MVP: `entities.description` column ships in the initial schema; composite `include` support and the odds derivation land with the topic page (F11–F13 in [prd-topic-lookup](../product/prd-topic-lookup.md)). v1: forecast surfaces activate per ADR-0012. No migration — greenfield.

## Related

- [../design/topic-lookup.md](../design/topic-lookup.md)
- [../design/query-api.md](../design/query-api.md)
- [../architecture/data-model.md](../architecture/data-model.md)
- [../product/prd-topic-lookup.md](../product/prd-topic-lookup.md)
- [0012-forecast-derived-data-module.md](0012-forecast-derived-data-module.md)
