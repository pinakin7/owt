---
type: adr
status: approved
owner: pinakin
summary: "Goldsky V2 datasets as canonical on-chain truth; legacy subgraphs prohibited."
tags: [area/ingestion, source/goldsky, source/polygon]
related:
  - ../architecture/data-sources.md
  - ../product/roadmap.md
---

# ADR-0007: Goldsky V2 datasets as canonical on-chain truth

## Context

Polymarket's CLOB V2 migration (2026-04-28) moved settlement to new exchange contracts with pUSD collateral, and the old public subgraphs are documented as **incomplete or incorrect** for post-migration data. Polymarket's indexing partner, Goldsky, publishes V2 datasets (`order_filled`, `orders_matched`, `user_balances`, `user_positions`) with historical replay. Self-indexing Polygon logs is possible but is real engineering that MVP doesn't need — the Data API's `/trades` already feeds the MVP tape.

## Decision

- From **v1**, Goldsky's Polymarket V2 datasets are the canonical source of on-chain execution facts (fills, positions, balances).
- **Legacy public subgraphs are prohibited data sources** anywhere in owt — a hard rule, enforced in review.
- A **direct Polygon RPC indexer** against the V2 exchange contracts is the documented fallback, reserved as an adapter-crate slot (`owt-source-polygon`) but unbuilt until needed.
- MVP ships with no on-chain ingestion at all.

## Consequences

- MVP scope shrinks and avoids a vendor negotiation; the trade tape is Data-API-sourced until v1.
- A vendor dependency enters at v1 — tier/pricing/dataset-name verification is open decision D-05 ([open-decisions](../governance/open-decisions.md)).
- Canonical fills carry block/tx keys so Data-API trades and Goldsky fills reconcile field-by-field ([normalization](../design/normalization.md)).
- Fill amounts are modeled as `amount_collateral` + `collateral_token` — pUSD-era naming, not `amount_usdc`.

## Alternatives considered

- **Legacy subgraphs** — prohibited: authoritative guidance says they are wrong post-V2; a research tool must not build on known-bad history.
- **Self-hosted Polygon indexer now** — rejected as premature engineering; preserved as the escape hatch.
- **No on-chain data ever** — rejected: v1's holder/fill analytics and execution-fact authority need it.

## Rollout notes

At v1 kickoff: verify Goldsky tier, pricing, and exact dataset names/schemas against live docs (they are report-sourced today), then close D-05.

## Related

- [../architecture/data-sources.md](../architecture/data-sources.md)
- [../product/roadmap.md](../product/roadmap.md)
