---
type: lld
status: draft
owner: pinakin
summary: "LLD for topic lookup: entity resolution flow, the composite entities read, the aggregated-odds algorithm, TUI screen behavior, and degraded modes."
tags: [area/api, area/tui, release/mvp]
related:
  - ../adr/0011-topic-lookup-entity-composite.md
  - query-api.md
  - search.md
  - forecasts.md
  - ../product/prd-topic-lookup.md
---

# Topic lookup

Serves F11–F14 in [prd-topic-lookup](../product/prd-topic-lookup.md): from any typed topic to one page of context, news, related markets, and predictions. Per [ADR-0011](../adr/0011-topic-lookup-entity-composite.md), a topic **is** an entity — this note specifies the composition, not a new data model.

## Resolution flow (query → entity)

1. `/topic <query>` (or an entity hit selected in `/v1/search`) debounces against the `entities` Typesense collection — same server-driven completion path the palette already uses ([tui-client](tui-client.md#command-palette--grammar), [search](search.md)).
2. Alias matches rank first (exact alias > normalized alias > name prefix), then `linked_market_count` breaks ties — "fed" completes to *Federal Reserve*, not an obscure person.
3. Accepting a completion routes to `TopicView` with the `ent_…` id. A query with no entity match falls through to federated search results — lookup never dead-ends.

## Composite read

One request paints the screen, mirroring the market-detail composite ([query-api](query-api.md#composite-screen-loads)):

```
GET /v1/entities/{id}?include=markets,events,news:10,odds
```

| Include | Content | Source |
|---|---|---|
| *(base)* | Entity: name, kind, `description`, alias list, link counts | `entities` + `entity_aliases` |
| `markets` | linked market summaries (state header fields), liquidity-ranked, cap 50 | `entity_links` ⋈ `markets` ⋈ `market_state` |
| `events` | linked event summaries, cap 20 | `entity_links` ⋈ `events` |
| `news:N` | latest linked NewsItems, cursor for more | GIN `news_items(entities)` |
| `odds` | the `EntityOdds` document (below) | derived on read (MVP) |

Server-side budget: composite ≤ 250 ms p95 warm (one transaction, parallel queries; the odds derivation is one indexed join over ≤ 50 member markets). `Cache-Control: max-age=5` like other reference reads.

## Aggregated odds (`EntityOdds`)

The core rule: **markets asking different questions are never averaged.** An entity maps to N markets across M events; a single blended number across them would be semantically wrong.

Algorithm (MVP, computed on read in `owt-api`):

1. Collect linked markets; drop `status != active` and stale members (`market_state.updated_at` older than the staleness TTL, default 24 h) — dropped ids are reported in `stale_members`.
2. **Group by `event_id`.** Each group is one *facet*: the event title, its member markets, and their outcome prices (implied probabilities) straight from `market_state`.
3. Rank facets by `max(liquidity_score) × recency` and cap at 10 for the composite (full list behind the cursor).
4. **Same-proposition merge:** only markets flagged as duplicate propositions (same `condition_id`, or a curated `same_as` entity-link annotation — rare intra-venue, the seam for multi-venue later) merge into one consensus number: weighted mean with `weight = liquidity_score` (fallback `volume_24h`), reported with dispersion (min/max, weighted stddev).
5. Multi-outcome markets normalize their outcome vector to sum to 1 before display; the facet shows per-outcome probabilities.

```json
{
  "entity_id": "ent_federal-reserve",
  "computed_at": "2026-07-09T10:14:02Z",
  "facets": [
    {
      "event_id": "678", "event_title": "Federal Reserve September Meeting",
      "markets": [
        { "market_slug": "will-fed-cut-rates-in-september",
          "outcomes": [ { "name": "Yes", "probability": 0.42 }, { "name": "No", "probability": 0.58 } ],
          "liquidity_score": 8.1, "updated_at": "2026-07-09T10:14:00Z", "stale": false }
      ],
      "consensus": null,
      "dispersion": null
    }
  ],
  "stale_members": []
}
```

`consensus`/`dispersion` are non-null only after a same-proposition merge. The DTO is normative in `owt-api-types`. Config knobs: `odds.staleness_ttl` (24 h), `odds.liquidity_floor` (facets below it render with a low-liquidity badge, never hidden), `odds.max_facets`.

**v1:** the same algorithm moves into `owt-forecast`, materialized as the `entity_odds` snapshot and pushed on WS `entity:{id}:odds` ([forecasts](forecasts.md#aggregated-odds-materialization-v1)). The read path switches from computing to serving the snapshot; the DTO does not change.

## Live updates & TUI behavior

- **MVP:** `TopicView` subscribes to `market:{slug}:state` for the visible facet members (registry-refcounted like any pane); state ticks patch facet rows locally. No new WS topics.
- **v1:** one `entity:{id}:odds` subscription replaces per-member bookkeeping for the odds board; the forecast pane subscribes via `market:{slug}:forecasts` of pinned members ([forecasts](forecasts.md#api-surface-additive-v1)).
- Layout: header (name, kind, description, link counts) over `[odds board | linked news | related markets/events]`, with a v1 forecast strip; breakpoints and golden frames follow the pane system in [tui-client](tui-client.md#screens-panes-layout).
- Forecast values render with model label + version and the disclosure line, always ([ADR-0012](../adr/0012-forecast-derived-data-module.md)).

## Degraded & empty states

| Condition | Behavior |
|---|---|
| entity has zero linked markets | honest empty state: entity header + news pane + "no linked markets yet" — never a blank screen |
| Typesense down | completion falls back to `entity_aliases` prefix match via Postgres ([search § degradation](search.md#degradation)) |
| all members stale | odds board renders with stale badges + ages; `stale_members` lists them |
| forecasts absent (MVP, or module down) | forecast strip shows "forecasts arrive in v1" / age-badged last values — same stale discipline as market panes |

## Test plan

- **Golden odds fixtures:** seed dataset in `owt-testkit` with known link topology → checked-in `EntityOdds` JSON; any algorithm change diffs deliberately.
- **Property tests:** a merge never mixes markets from different events; facet probabilities stay in [0,1] and multi-outcome vectors sum to 1 ± ε; excluded stale members never contribute weight.
- **Contract test:** composite `include` combinations against a seeded store; empty-entity case included.
- **Golden frames:** `TopicView` per breakpoint, including the empty and all-stale states.

## Related

- [../adr/0011-topic-lookup-entity-composite.md](../adr/0011-topic-lookup-entity-composite.md)
- [query-api.md](query-api.md)
- [search.md](search.md)
- [forecasts.md](forecasts.md)
- [../product/prd-topic-lookup.md](../product/prd-topic-lookup.md)
