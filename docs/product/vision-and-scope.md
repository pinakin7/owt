---
type: product
status: draft
owner: pinakin
summary: "Why owt exists: the jobs it does, who it serves, its boundaries per release, and what it will never be."
tags: [area/product, release/mvp]
related:
  - prd-mvp.md
  - roadmap.md
  - ../architecture/system-overview.md
  - ../adr/0002-ratatui-tui-first-client.md
  - ../adr/0008-read-only-through-v1.md
---

# owt — vision and scope

## The pitch

**owt** is an open-source research terminal for prediction markets. It fuses Polymarket's official market data, on-chain execution facts, and the surrounding evidence — news, RSS, comments, and later social signals — into one queryable, replayable timeline, surfaced through a fast native terminal UI.

Prediction market prices are the world's most honest headlines; owt is the terminal for reading them. The name is Northern English dialect: *owt* means "anything" — as in, anything moving the markets. (Working name; branding is tracked in [open-decisions](../governance/open-decisions.md).)

## The problem

When a market jumps eleven cents, the *why* is scattered: the venue UI shows the price but not the cause; the news that moved it is on a publisher site; the chatter is on X; the actual fills are on Polygon. Anyone doing serious research — an analyst, a journalist, a trader doing post-mortems — reassembles that picture by hand, tab by tab, with no history and no replay. Bloomberg solved this for securities decades ago; nothing open does it for prediction markets.

## The four jobs

1. **Discover** — find markets and events worth attention: instant search, tag and liquidity filters, watchlists, saved views.
2. **Explain** — link price action to evidence: a unified per-event timeline that interleaves price moves, trades, news items, and comments, with correlation confidence.
3. **Pivot** — go from a market to its full evidence trail in seconds: order book, trade tape, linked articles, on-chain fills, and back out to related markets.
4. **Look up** — start from the world, not the market: type any topic and get its markets, aggregated odds, news, and (v1) model forecasts on one page ([prd-topic-lookup](prd-topic-lookup.md)).

## Who it's for

| Persona | What they need from owt |
|---|---|
| **Markets researcher / analyst** | Post-mortems: "what moved this market, when, and on what evidence" — with export |
| **Active prediction-market trader** | Live books, tape, and alerts on moves/news bursts across a watchlist |
| **Journalist / OSINT researcher** | Event timelines with citable sources; markets as a lens on breaking news |
| **Data tinkerer / quant hobbyist** | A clean, self-hostable indexed dataset and a scriptable API under the terminal |

## What owt is

An **intelligence layer above the official data plane** — Polymarket-first, multi-source, self-hostable. The server (`owtd`) ingests, normalizes, stores, and indexes; the terminal client (`owt`) is a native TUI ([ADR-0002](../adr/0002-ratatui-tui-first-client.md)). Everything is open source under Apache-2.0.

## What owt is not

- **Not a trading engine** and not an order router — Polymarket's CLOB already exists; owt reads it.
- **No custody, ever, by default** — read-only through v1 ([ADR-0008](../adr/0008-read-only-through-v1.md)); any later execution-adjacent feature is opt-in and signer-isolated.
- **Not a data-plane rebuild** — official APIs and sanctioned datasets are the source of truth; owt adds fusion, history, and workflow.
- **Not an HFT tool** — latency targets are "terminal-grade," sub-second, not microseconds.
- **Not a Polymarket UI clone** — if a feature merely mirrors the official site or the official CLI, it is not roadmap-worthy.

## Differentiation

Polymarket's official UI answers "what is the price"; its experimental Rust CLI answers "let me script an order." owt answers **"why is the price, and what happened around it"**:

- cross-source indexing (markets + news + comments + on-chain) under one query surface
- topic-first lookup: from any real-world topic to its markets, aggregated odds, and evidence in one page
- event-centric timelines with price/news correlation
- owt's own statistical (non-LLM) model outputs — trend, volatility, divergence, calibration-adjusted probability — informational only, always disclosed ([ADR-0012](../adr/0012-forecast-derived-data-module.md))
- alerts on moves, spreads, news bursts, and comment spikes
- watchlists and saved research views that persist
- replayable ingestion — the evidence trail can be reconstructed, byte-for-byte, later
- operator-grade observability, because a research tool you can't trust to be ingesting is worse than none

## Boundaries by release

| Release | Boundary |
|---|---|
| **MVP** | Read-only terminal: discovery, search, market/event detail, topic lookup with aggregated odds, live market-channel data, timelines, watchlists, news linkage. No accounts. |
| **v1** | Realtime everywhere (comments, sports, on-chain fills), alert engine, correlation, model forecasts on topic and market pages. Still no accounts. |
| **v2** | SIWE identity, private workspaces, saved research sharing, Polymarket account *views*. Web client track may open. |
| **v3** | Execution-adjacent: guarded order preview, isolated local signer, audit trail. Gated by new ADRs. |

Full deliverables and gates: [roadmap](roadmap.md).

## Success criteria

- **Time-to-why:** from "why did this market move?" to a linked evidence trail in under 60 seconds inside the TUI.
- **Time-to-run:** clone → `docker compose up` → seeded terminal in under 15 minutes on a laptop.
- **Trustworthy ingest:** a self-hoster can see at a glance (status bar + dashboards) that data is fresh, and replay any window deterministically.
- **Community pull:** external contributors ship new source adapters without touching core crates — the adapter seam is the plugin API.
- **Honest scope:** zero custody incidents because there is nothing to steal through v1.

## Related

- [prd-mvp.md](prd-mvp.md)
- [roadmap.md](roadmap.md)
- [../architecture/system-overview.md](../architecture/system-overview.md)
- [../adr/0002-ratatui-tui-first-client.md](../adr/0002-ratatui-tui-first-client.md)
- [../adr/0008-read-only-through-v1.md](../adr/0008-read-only-through-v1.md)
