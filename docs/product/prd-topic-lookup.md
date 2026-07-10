---
type: product
status: draft
owner: pinakin
summary: "Product requirements for the topic-lookup flagship: the topic page, search-first flow, aggregated market odds, and the v1 model-forecast pane."
tags: [area/product, release/mvp]
related:
  - prd-mvp.md
  - vision-and-scope.md
  - ../design/topic-lookup.md
  - ../design/forecasts.md
  - ../adr/0011-topic-lookup-entity-composite.md
---

# PRD — Topic lookup & predictions

The flagship layer on top of the MVP terminal ([prd-mvp](prd-mvp.md)): type **any real-world topic** — "Fed rate cut", "Bitcoin", "US election" — and get one page answering *what's known, what's related, and what the markets (and owt's models) predict*. This is the fourth job in [vision-and-scope](vision-and-scope.md): **Look up** — start from the world, not from a market.

A topic is an entity ([ADR-0011](../adr/0011-topic-lookup-entity-composite.md)); predictions are aggregated market odds (MVP) plus owt's statistical model outputs (v1, [ADR-0012](../adr/0012-forecast-derived-data-module.md)). No LLM features anywhere in this spec.

## Users and jobs

All four personas in [vision-and-scope](vision-and-scope.md) start from a question about the world more often than from a market slug:

| Persona | Topic-lookup answer |
|---|---|
| Researcher / analyst | one page per topic: every related market, its odds, and the news trail — exportable |
| Trader | fastest route from "something happened to X" to the tradable markets on X, with staleness honesty |
| Journalist / OSINT | "what do prediction markets say about X" with citable market links and evidence |
| Quant hobbyist | model signals per market/topic via the same API the page uses |

## Feature set

Feature IDs continue [prd-mvp](prd-mvp.md)'s F-series. Endpoints: [query-api](../design/query-api.md); mechanics: [topic-lookup](../design/topic-lookup.md).

### F11 — Topic page (MVP)
One screen per entity: context header (name, kind, curated description, link counts), the aggregated-odds board, linked news, related markets and events.
**Data:** entity dictionary + `entity_links`, `market_state`, linked news — all MVP-planned ingest; one composite `GET /v1/entities/{id}?include=markets,events,news:10,odds`.
**AC:** `/topic bitcoin` renders header + ≥ 1 odds facet + news pane in ≤ 400 ms on the seed dataset; an entity with zero linked markets shows an honest empty state (header + news + notice), never a blank screen; visible facet rows tick live via existing market state subscriptions.

### F12 — Topic search-first flow (MVP)
`/topic <query>` palette command with entity-boosted typeahead; entity results in federated search route to the topic page on Enter.
**Data:** `entities` Typesense collection (+ `description`, activity signal — [search](../design/search.md)).
**AC:** "fed" completes to *Federal Reserve* above any person entity on the seed dataset; a query with no entity match falls through to federated search results, never a dead end; completion renders < 150 ms after keystroke pause.

### F13 — Aggregated market odds (MVP)
The odds board: linked markets grouped by event into facets; implied probabilities from live state; same-proposition markets merged into a liquidity-weighted consensus with dispersion; staleness handled honestly.
**Data:** `entity_links ⋈ market_state`, derived on read ([topic-lookup § odds](../design/topic-lookup.md#aggregated-odds-entityodds)).
**AC:** markets from different events are never averaged (property-tested); a member whose state exceeds the staleness TTL renders with a stale badge and contributes zero weight to any merge; multi-outcome facets display probabilities summing to 1 ± ε.

### F14 — Model forecasts on the topic page (v1)
A forecast strip rendering `owt-forecast` outputs (trend, volatility, volume anomaly, book imbalance, divergence) for the topic's markets, each labeled with model id + version, kind (`signal`/`forecast`), and the research disclosure.
**Data:** `forecast_state` via `GET /v1/markets/{slug}/forecasts` and `GET /v1/entities/{id}/forecasts`; live via WS `market:{slug}:forecasts` / `entity:{id}:odds` ([forecasts](../design/forecasts.md)).
**AC:** every rendered value shows its model label and version; responses carry the disclosure string and the TUI displays it on the strip; when the forecast module is down the strip degrades to age-badged last values, panes never blank; in MVP builds the strip shows "forecasts arrive in v1".

## Commands

| Command | Effect |
|---|---|
| `/topic <query>` | open topic page (completes from entities) |
| `/open <entity>` | existing verb — entity completions route to the topic page too |

## Screen (wireframe)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ owt  topic: Federal Reserve (org)                          LIVE  ws: ok  │
├──────────────────────────────────────────────────────────────────────────┤
│ US central bank. 14 markets · 6 events · 212 news items                  │
├───────────────────────────────┬──────────────────────────────────────────┤
│ Odds                          │ Linked News                              │
│ Sept Meeting                  │ Reuters: Fed official signals caution…   │
│  cut 25bp   YES 42¢ ▲         │ FT: Treasury yields fall on…             │
│  cut 50bp   YES  8¢           │ AP: Powell remarks at…                   │
│ Dec Meeting                   ├──────────────────────────────────────────┤
│  any cut    YES 71¢           │ Related                                  │
│  (stale 26h ⚠)                │ mkt will-fed-cut-rates-in-september      │
│                               │ evt federal-reserve-september-meeting    │
├───────────────────────────────┴──────────────────────────────────────────┤
│ Forecasts (v1): momentum +0.4 · rvol 12% · divergence 0.02  [model v1.0] │
│ Statistical model output for research purposes; not financial advice.    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Data requirements

| Pane | Endpoint | Upstream source | Freshness target |
|---|---|---|---|
| Topic composite | `GET /v1/entities/{id}?include=…` | entity dictionary + Gamma/CLOB state + news links | composite ≤ 250 ms p95; odds staleness TTL 24 h |
| Odds board live | WS `market:{slug}:state` (MVP) / `entity:{id}:odds` (v1) | CLOB WS → state | ≤ 750 ms p95 live |
| Forecast strip *(v1)* | `…/forecasts` + WS `market:{slug}:forecasts` | `owt-forecast` derived data | nowcast lag ≤ 5 s p95 |

## Rollout

| Train | Ships |
|---|---|
| **MVP** | F11, F12, F13 (odds derived on read); forecast schema slots reserved |
| **v1** | F14: `owt-forecast` signals, forecast/odds endpoints + WS topics, materialized odds |
| **v2** | calibration-adjusted probability (`calib-prob.v1`) + backtest/evaluation harness |

## Out of scope

- **LLM-generated summaries, outlooks, or probabilities** — excluded by [ADR-0012](../adr/0012-forecast-derived-data-module.md); entity descriptions are human-curated.
- **Cross-venue odds** — the same-proposition merge is the designed seam, but only Polymarket exists through v1 ([roadmap § beyond v3](roadmap.md#beyond-v3)).
- **Trading or advice** — read-only product ([ADR-0008](../adr/0008-read-only-through-v1.md)); everything rendered carries the research disclosure.
- **Auto-created entities** — the dictionary stays curated; ungoverned entity growth degrades linking precision ([normalization](../design/normalization.md#entity-resolution-mvp-dictionary--rules)).

## Related

- [prd-mvp.md](prd-mvp.md)
- [vision-and-scope.md](vision-and-scope.md)
- [../design/topic-lookup.md](../design/topic-lookup.md)
- [../design/forecasts.md](../design/forecasts.md)
- [../adr/0011-topic-lookup-entity-composite.md](../adr/0011-topic-lookup-entity-composite.md)
