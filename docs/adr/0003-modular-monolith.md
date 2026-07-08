---
type: adr
status: approved
owner: pinakin
summary: "Modular monolith with async workers instead of microservices."
tags: [area/architecture]
related:
  - ../architecture/system-overview.md
  - ../architecture/cargo-workspace.md
---

# ADR-0003: Modular monolith with async workers

## Context

The system's complexity is already external: a dozen upstream surfaces (three REST APIs, two WebSocket families, on-chain datasets, feeds) that drift, throttle, and disconnect independently. Splitting our *own* side into networked microservices would add deployment, versioning, and debugging cost without improving correctness — and owt must stay trivially self-hostable by one person on one machine.

## Decision

One server binary, **`owtd`**, containing all backend modules, with **role flags** selecting what a given instance runs:

```
owtd serve --roles ingest,normalize,index,api,alerts   # everything (default)
owtd serve --roles api                                  # scale-out later, same binary
```

Modules are separate crates with strictly layered dependencies ([cargo-workspace](../architecture/cargo-workspace.md)); workers are supervised tokio tasks; all cross-module data flow goes through NATS JetStream subjects ([realtime-bus](../design/realtime-bus.md)), so processes can be split later without code changes.

## Consequences

- One artifact to build, release, and roll back; `docker compose up` runs the whole backend.
- Scale-out is configuration (role-scoped instances sharing bus + store), not a rewrite.
- Module boundaries are enforced by the crate graph at compile time instead of by network contracts at runtime.
- A crashing module can take down co-located roles — mitigated by task supervision and, when it matters, role isolation.

## Alternatives considered

- **Microservices-by-default** — rejected: premature; multiplies ops burden for a self-hosted OSS tool.
- **Serverless/functions** — rejected: long-lived WebSocket consumers and stateful backfill don't fit.
- **Unstructured single binary** (no crate layering) — rejected: loses the compile-time boundaries that make the monolith modular.

## Rollout notes

None (greenfield). Revisit only when a named bottleneck demands isolating a role onto dedicated instances — which this design already permits.

## Related

- [../architecture/system-overview.md](../architecture/system-overview.md)
- [../architecture/cargo-workspace.md](../architecture/cargo-workspace.md)
