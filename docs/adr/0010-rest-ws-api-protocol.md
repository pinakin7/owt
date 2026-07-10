---
type: adr
status: approved
owner: pinakin
summary: "REST/JSON plus one multiplexed WebSocket as the client API; gRPC and GraphQL rejected."
tags: [area/api]
related:
  - ../design/query-api.md
  - ../architecture/cargo-workspace.md
---

# ADR-0010: REST + WebSocket as the client API protocol

## Context

The research report left the query layer as a "GraphQL-or-REST facade." The client reality settles it: the first-party client is a Rust TUI sharing a types crate with the server, power users will script against the API with curl and jq, and a future web client should reuse the surface unchanged.

## Decision

- **REST/JSON over HTTP** for request/response, versioned under `/v1`, additive-only within a major version. OpenAPI is generated (utoipa) for third-party consumers.
- **One multiplexed WebSocket** per client for live data: subscribe/unsubscribe frames carrying topic strings; typed server frames; server-side conflation ([query-api](../design/query-api.md)).
- The contract is the **`owt-api-types` crate** — DTOs, error codes, WS frames — shared by server, client SDK (`owt-client`), and TUI. No codegen pipeline.
- The TUI never consumes NATS directly: internal bus subjects stay private and unstable; the API is the only public contract.

## Consequences

- Type safety where it matters (first-party Rust ↔ Rust) at zero toolchain cost; curl-ability for everyone else.
- The WS sub-protocol must be explicitly versioned and tested for resume semantics — specified in [query-api](../design/query-api.md).
- A future web client consumes the identical surface; nothing is client-specific.

## Alternatives considered

- **gRPC** — codegen buys nothing when both ends share a Rust crate; loses curl-ability; a later web client would drag in a grpc-web proxy.
- **GraphQL** — resolver/N+1/caching machinery and schema governance without the third-party client diversity that justifies it.
- **Direct NATS access for the TUI** — freezes internal subjects into a public contract, bypasses future authz, and exposes the bus to the network; explicitly banned (power users may still read the bus on their own boxes — unsupported).

## Rollout notes

Contract tests between `owt-client` and `owt-api` run from the first code phase; the WS protocol carries a version field from day one.

## Related

- [../design/query-api.md](../design/query-api.md)
- [../architecture/cargo-workspace.md](../architecture/cargo-workspace.md)
