---
type: product
status: draft
owner: pinakin
summary: "Testable product requirements for the owt MVP: features, screens, commands, data needs, NFRs, and acceptance criteria."
tags: [area/product, release/mvp]
related:
  - vision-and-scope.md
  - roadmap.md
  - ../design/tui-client.md
  - ../design/query-api.md
---

# PRD — MVP

The MVP is a **read-only research terminal**: one self-hostable server (`owtd`), one native TUI (`owt`), Polymarket market data (historical + live market channel) fused with news. No accounts, no trading ([ADR-0008](../adr/0008-read-only-through-v1.md)).

## Users and jobs

Personas in [vision-and-scope](vision-and-scope.md). The MVP serves three jobs end-to-end:

| Job | MVP answer |
|---|---|
| "Find markets worth watching" | instant search + filters, watchlists, saved views |
| "What is this market doing right now?" | market detail: live book, tape, price chart, state |
| "Why did it move?" | event timeline interleaving price moves and linked news |
| "What do markets say about X?" | topic page: entity context, aggregated odds, linked news (F11–F13, [prd-topic-lookup](prd-topic-lookup.md)) |

## Feature set

Each feature lists its data dependencies and acceptance criteria (AC). Endpoints refer to [query-api](../design/query-api.md); sources to [data-sources](../architecture/data-sources.md).

### F1 — Instant search
Typo-tolerant search-as-you-type over markets, events, and news, with a filter DSL (`tag:politics liquidity:>10000 status:active free text`).
**Data:** Typesense collections fed from Gamma + news ingest.
**AC:** results render < 150 ms after keystroke pause on the demo dataset; DSL filters compose with free text; zero-result state suggests loosening filters.

### F2 — Market detail
One screen per market: order book (top 5 levels each side), trade tape, price sparkline/candles, market state (last/bid/ask/spread/24h volume/liquidity), linked news pane, event context.
**Data:** CLOB book + price history, Data API trades, Gamma metadata, live `market` WS channel, news links.
**AC:** all panes populate for any active demo market; live updates visibly tick without flicker at ≤10 Hz; degraded mode (WS down) shows stale badges with age, not blank panes.

### F3 — Event detail and timeline
One screen per event: member markets with prices, plus the merged timeline (price moves, trades bursts, news items) newest-first with kind markers and confidence.
**Data:** `timeline_items` via `/v1/events/{slug}/timeline`.
**AC:** a price move ≥ 3¢ within 30 min on a demo market appears as a timeline item with `numeric_delta`; adjacent news items within the correlation window link to it.

### F4 — Watchlists
Pin markets into named watchlists; a watchlist screen shows compact live rows (price, Δ24h, spread, volume).
**Data:** workspace tables (server-side, `default` workspace); live state via WS subscription per visible row.
**AC:** watchlists survive client restart and server restart; adding/removing reflects in < 1 s.

### F5 — Saved views
Save a named (screen, query, layout) combination; reopening restores it exactly.
**AC:** a saved event-workspace view restores query, sort, and pane layout byte-identically.

### F6 — Market–news linkage
News items carry `market_links`/`event_links` from entity resolution; market detail shows its linked news; news items list their linked markets.
**Data:** RSS/GDELT ingest → normalization → entity linking.
**AC:** a demo news item about a tracked entity appears in the linked-news pane of the related market within 2 min of feed publication.

### F7 — Historical backfill
On first run, `owtd backfill` builds the market universe and history: Gamma events/markets, CLOB price history for active tokens, recent Data API trades, news history for configured feeds.
**AC:** full backfill (universe + 30 days of price history for top-N markets) completes ≤ 4 h on a laptop within configured rate budgets; interrupted backfill resumes from checkpoints without duplicates.

### F8 — Live market data
The Polymarket `market` WS channel streams book/price/last-trade into the bus, store, and subscribed TUI panes.
**AC:** tick-to-pane p95 ≤ 750 ms; after a forced 60 s disconnect, the client resyncs (snapshot + deltas) with no permanent gap and a visible reconnect indicator.

### F9 — Command palette and keyboard-first navigation
Palette-first commands with completion; vim-ish navigation everywhere; `?` help overlay.
**AC:** every mouse-free flow in the demo script works; commands complete from search; unknown command shows help hint.

### F10 — Export
`/export` writes the current pane's data (timeline, tape, watchlist) to JSON or CSV.
**AC:** exported timeline reimports as valid JSON; file path is printed and correct.

### F11–F13 — Topic lookup (MVP slice)
The flagship topic page — `/topic <query>` resolves any real-world topic to its entity and renders context, aggregated market odds, and linked news in one composite read. Full spec, ACs, and the v1 forecast pane (F14): [prd-topic-lookup](prd-topic-lookup.md).

## Commands (MVP grammar)

| Command | Effect |
|---|---|
| `/open <market-or-event>` | open detail screen (completes from search) |
| `/topic <query>` | open topic page (completes from entities — [prd-topic-lookup](prd-topic-lookup.md)) |
| `/watch <market>` / `/pin` | add to watchlist / pin pane |
| `/compare <m1> <m2>` | side-by-side market panes |
| `/news <query>` | news search scoped to current context |
| `/export [json\|csv] [path]` | export current pane |
| `/view save\|open <name>` | saved views |
| `/help`, `?` | help overlay |
| `/alert …` | **v1** — parses but responds "alerts arrive in v1" in MVP |

Full grammar (EBNF) and keybindings: [tui-client](../design/tui-client.md).

## Screen inventory

1. **Search/browse** — palette + results table + preview pane.
2. **Market detail** — wireframe below.
3. **Event workspace** — wireframe below.
4. **Watchlists** — live rows + jump-to-detail.
5. **Help overlay** — keys + commands.
6. **Topic view** — entity header + odds board + linked news + related ([prd-topic-lookup](prd-topic-lookup.md)).

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ owt  market: will-fed-cut-rates-in-september            LIVE   ws: ok    │
├──────────────────────────────────────────────────────────────────────────┤
│ Price 42.0¢ YES   Spread 2.0¢   Vol 24h $3.2M   Liquidity high           │
│ Event: Federal Reserve September Meeting    Tags: economics fed macro    │
├──────────────────┬────────────────────────────┬──────────────────────────┤
│ Order Book       │ Trade Tape                 │ Linked News              │
│ ask 0.43  1,200  │ 10:14:02  BUY  0.42   90   │ Reuters: Fed official…   │
│ ask 0.44    800  │ 10:13:58  SELL 0.41  120   │ FT: Treasury yields…     │
│ mid 0.42         │ 10:13:44  BUY  0.42   50   │ AP: Powell remarks…      │
│ bid 0.41  1,050  │ …                          │ …                        │
│ bid 0.40  2,100  │                            │                          │
├──────────────────┴────────────────────────────┴──────────────────────────┤
│ Timeline: Powell speech → +3.0¢ in 18m → 12 linked articles              │
└──────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ owt  workspace: election-night                                           │
├──────────────────────────────────────────────────────────────────────────┤
│ Query: tag:politics AND country:US AND liquidity:high                    │
├────────────────────────────┬─────────────────────────────────────────────┤
│ Markets                    │ Event timeline                              │
│ - Presidential winner      │ 18:03 AP calls state X                      │
│ - Electoral margin         │ 18:07 market A +11¢                         │
│ - Senate control           │ 18:12 comment spike (v1)                    │
│ - House control            │ 18:14 Reuters bulletin                      │
├────────────────────────────┴─────────────────────────────────────────────┤
│ /watch /pin /compare /news /export /view /open <slug>                    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Data requirements per pane

| Pane | Endpoint | Upstream source | Freshness target |
|---|---|---|---|
| Search results | `GET /v1/search` | Typesense (Gamma + news) | index lag ≤ 5 s |
| Market state header | `GET /v1/markets/{slug}` + WS `market:{slug}:state` | Gamma + CLOB + WS | ≤ 750 ms p95 live |
| Order book | `…/book` + WS `market:{slug}:book` | CLOB + WS | conflated 4–10 Hz |
| Trade tape | `…/trades` + WS `market:{slug}:trades` | Data API + WS | ≤ 750 ms p95 live |
| Price chart | `…/prices?resolution=…` | CLOB history → candles | backfilled + 1m rollups |
| Linked news | `…/news` | RSS/GDELT via entity links | ≤ 2 min from publication |
| Event timeline | `/v1/events/{slug}/timeline` | timeline builder | ≤ 2 min for news; ≤ 5 s for price items |
| Watchlist rows | `GET /v1/watchlists/{id}` + WS `watchlist:{id}` | store + WS | ≤ 1 s |
| Topic page | `GET /v1/entities/{id}?include=…` + WS `market:{slug}:state` per facet | entity links + market_state + news | composite ≤ 250 ms p95; odds staleness TTL 24 h |

## Non-functional requirements (proposed — confirm before feature-freeze)

Full SLO tables: [observability](../ops/observability.md). Headlines the MVP is accepted against:

- **Freshness:** WS tick → pane p95 ≤ 750 ms; news → searchable p95 ≤ 2 min.
- **Query latency (server, warm):** search ≤ 100 ms p95; market-detail composite ≤ 150 ms p95; candles ≤ 250 ms p95.
- **Client:** input echo ≤ 50 ms; frame ≤ 16 ms at 200×60; cold start ≤ 2 s; RSS ≤ 150 MB with 10 live subscriptions; min terminal 80×24 (degrades gracefully, never corrupts).
- **Capacity:** ≤ 50 concurrent TUI clients per `owtd`; burst 500–2,000 canonical msgs/s without loss (acked bus messages survive crash).
- **Operability:** single `docker compose up` dev stack; `owtd` roles splittable later without config redesign.

## Out of scope (MVP)

Auth/accounts/PII; trading and Polymarket account views; alert engine (parses, doesn't fire — v1); model forecasts (F14 — v1, schema slots reserved; [prd-topic-lookup](prd-topic-lookup.md)); RTDS comments, sports/user channels (v1); Goldsky on-chain fills (v1); X/social (v2); web client (≥ v2); mobile; notes/annotations (v2 candidate).

## Acceptance demo script

1. `docker compose up` → `owtd migrate` → `owtd backfill --top 200 --days 30` completes with checkpoint log.
2. Launch `owt`; status bar shows server + WS healthy.
3. Search "fed" → results < 150 ms; filter `tag:economics liquidity:>10000`.
4. `/open` top result → market detail fully populated; book/tape ticking live.
5. Kill network 60 s → stale badges + reconnect indicator; restore → resync, no gap.
6. Jump to event → timeline shows price-move items; a linked news item opens its detail.
7. `/topic federal reserve` → topic page renders odds facets, news, and related markets; a facet row ticks live.
8. `/watch` the market; restart `owt`; watchlist row is live.
9. `/view save fed-watch`; reopen restores layout+query.
10. `/export json` timeline → valid file.
11. `owtd` dashboard (Grafana) shows ingest lag, zero DLQ.

## Related

- [vision-and-scope.md](vision-and-scope.md)
- [roadmap.md](roadmap.md)
- [../design/tui-client.md](../design/tui-client.md)
- [../design/query-api.md](../design/query-api.md)
