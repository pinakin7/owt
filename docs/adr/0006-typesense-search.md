---
type: adr
status: approved
owner: pinakin
summary: "Typesense for instant search; OpenSearch deferred as the scale path."
tags: [area/search]
related:
  - ../design/search.md
---

# ADR-0006: Typesense for instant search

## Context

Search-as-you-type with typo tolerance is core terminal UX (F1 in [prd-mvp](../product/prd-mvp.md)): a researcher types three characters and expects ranked markets, events, and news in under 150 ms. The engine must be operable by a self-hoster (one container, sane defaults) and fully **rebuildable from Postgres** — search is a derived index, never a system of record.

## Decision

**Typesense**, as its own compose service:

- Collections `markets`, `events`, `news`, `entities` (v1: `comments`), schemas defined as code in `owt-search` ([search](../design/search.md)).
- Blue/green rebuilds via collection **alias swap**, always sourced from Postgres only.
- Entity aliases exported as Typesense synonyms so "FOMC" finds Fed markets.
- **Thin hand-rolled REST client** in `owt-search` — there is no mature official Rust client, so we own ~300 lines of HTTP instead of an unmaintained dependency; the server version is pinned in compose.

## Consequences

- Instant, typo-tolerant UX with near-zero tuning; one more service in the 4-service compose stack.
- Degradation path required and specified: if Typesense is down, `/v1/search` falls back to Postgres trigram/FTS with reduced features.
- API keys stay server-side; the TUI never talks to Typesense directly.

## Alternatives considered

- **Meilisearch** — genuinely comparable; Typesense chosen per the research report's operational recommendation and its first-class alias-swap rebuild story. Not a religious choice; revisiting would be cheap while the client is thin.
- **OpenSearch** — powerful aggregations, heavy ops; deferred as the named scale path if search becomes analytics-shaped.
- **Postgres FTS/trigram only** — kept as the degradation mode, but inadequate as the primary UX (typo tolerance, ranked federation).

## Rollout notes

None (greenfield). Collection schema changes bump a version and trigger an alias-swap rebuild ([search](../design/search.md)).

## Related

- [../design/search.md](../design/search.md)
