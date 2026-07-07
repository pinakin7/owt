# owt

**An open-source research terminal for prediction markets.**

Prediction market prices are the world's most honest headlines; owt is the terminal for reading them. It fuses Polymarket's official market data, on-chain execution facts, and the surrounding evidence — news, RSS, comments — into one queryable, replayable timeline, surfaced through a fast native terminal UI. When a market jumps eleven cents, owt answers *why*.

> *owt* — Northern English dialect for "anything." As in: anything moving the markets.

## Status

**Pre-alpha, docs-first.** This repository currently contains the project's founding documentation — product specification, architecture, and low-level designs. Code lands next, following the [roadmap](docs/product/roadmap.md).

## What it will do (MVP)

- **Instant search** across markets, events, and news with a filter DSL (`tag:politics liquidity:>10000`)
- **Market detail**: live order book, trade tape, price history, and linked news in one screen
- **Event timelines** that interleave price moves with the news that caused them
- **Watchlists and saved views** that persist across sessions
- **Self-hosted in minutes**: one `docker compose up`, one backfill command, one binary

Read-only by design through v1 — no wallets, no custody, no trading ([why](docs/adr/0008-read-only-through-v1.md)).

## Architecture at a glance

Rust everywhere: a modular-monolith server (`owtd`) ingests Polymarket APIs (Gamma, CLOB, Data, WebSockets), RSS, and news feeds through a replayable NATS JetStream pipeline into PostgreSQL/Timescale and Typesense; a native ratatui terminal client (`owt`) consumes a REST + WebSocket API. Every derived store is rebuildable; every raw payload is replayable.

Start here: [system overview](docs/architecture/system-overview.md) · [data sources](docs/architecture/data-sources.md) · [full documentation vault](docs/README.md)

## Documentation

The `docs/` directory is an [Obsidian](https://obsidian.md)-compatible vault of atomic, interlinked notes — open it as a vault to browse the relation graph, or read it straight on GitHub. The [vault index](docs/README.md) maps every document.

## Security

No custody, no keys, no PII through v1; the API binds to localhost by default. Details: [security & privacy](docs/ops/security-and-privacy.md). To report a vulnerability, see [SECURITY.md](SECURITY.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The best early contributions are source adapters and doc review — the adapter seam is designed to be the plugin API.

## License

Code: [Apache-2.0](LICENSE). Documentation (`docs/`): CC-BY-4.0.
