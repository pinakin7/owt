---
type: adr
status: approved
owner: pinakin
summary: "Rust for all backend services, superseding the research report's recommended stack."
tags: [area/architecture]
related:
  - 0002-ratatui-tui-first-client.md
  - ../architecture/cargo-workspace.md
---

# ADR-0001: Rust for all backend services

## Context
The workload is a research terminal — sustained ingestion from a dozen upstream feeds, normalization, fanout, and sub-second query serving — not HFT. But the language choice must also carry the v3 execution-adjacent path (signer isolation, order preview), where predictable latency and memory safety both matter. Polymarket itself ships official Rust tooling (an experimental CLI and SDK direction), so the ecosystem gravity for this domain is real.

## Decision

Rust for every service in the system, and (with [ADR-0002](0002-ratatui-tui-first-client.md)) for the client too. Baseline ecosystem choices, revisited only via new ADRs:

- **Runtime:** tokio (multi-thread), tower middleware ecosystem
- **HTTP/WS server:** axum; OpenAPI generation via utoipa
- **HTTP/WS clients:** reqwest and tokio-tungstenite, built on rustls (no OpenSSL linkage in distributed binaries)
- **Database access:** sqlx with compile-time-checked SQL (Timescale features need raw SQL regardless)
- **Serialization:** serde/serde_json; **rate limiting:** governor; **bus client:** async-nats
- **Errors:** thiserror in library crates, anyhow in binaries
- Workspace layout and dependency layering: [cargo-workspace](../architecture/cargo-workspace.md)

## Consequences

- One language across daemon, client, SDK, and test tooling; API contracts are a shared crate (`owt-api-types`), not generated bindings.
- Memory safety without GC pauses; single static binaries make TUI distribution trivial (cargo-dist).
- Compile times and a steeper learning curve are the tax; CI caching and a small crate graph keep it tolerable.
- The report's original managed-runtime-stack rationale is void; the research report itself has been removed from the vault, and this ADR (with [ADR-0002](0002-ratatui-tui-first-client.md)) is the sole surviving record of that rejected alternative.
- Contributor pool skews systems/backend — acceptable for an infrastructure-heavy OSS project.

## Alternatives considered

- **A managed-runtime backend with a matching browser-compiled client toolchain** (the report's recommendation) — strong typed domain modeling, but its headline benefit (one codebase spanning a browser UI) evaporates with a TUI-first client, and it is not the low-latency lineage the founder asked for.
- **C++** — the classic ultra-low-latency choice; rejected for slower development, a weaker web-service ecosystem, more dangerous contribution surface for an OSS project, and no memory safety.
- **TypeScript/Node** — largest contributor pool; rejected as not remotely low-latency-pedigreed and weaker for long-lived ingestion daemons.
- **Go** — fine for services; rejected because the founder's directive pointed at the ULL tier, and Rust's type system is a better home for the canonical domain model.

## Rollout notes

Greenfield — no migration. First code phase scaffolds the workspace exactly as [cargo-workspace](../architecture/cargo-workspace.md) specifies.

## Related

- [0002-ratatui-tui-first-client.md](0002-ratatui-tui-first-client.md)
- [../architecture/cargo-workspace.md](../architecture/cargo-workspace.md)
