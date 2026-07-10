---
type: adr
status: approved
owner: pinakin
summary: "Read-only product through v1: no wallets, accounts, or trading until v2/v3."
tags: [area/product, area/security]
related:
  - ../product/roadmap.md
  - ../ops/security-and-privacy.md
  - ../product/vision-and-scope.md
---

# ADR-0008: Read-only through v1

## Context

Polymarket's public surfaces (Gamma, CLOB market data, Data API, WebSockets) are open and rich; its authenticated flows are not casual — L1 auth signs EIP-712 messages with the wallet's private key, L2 uses HMAC API credentials, and order creation always requires user-side signing. Touching keys means custody-grade security engineering and a very different compliance posture. Meanwhile the product's core value — explaining why markets move — needs none of that.

## Decision

Through v1, owt ships **no wallets, no user accounts, no custody, and no trading**. Concretely:

- MVP/v1 collect no PII; the API binds to localhost by default with an optional static bearer token for remote self-hosts.
- SIWE-based workspaces and private account views arrive at **v2**, gated by a new auth ADR (including the terminal device-flow design).
- Execution-adjacent features (guarded order preview, local signer) arrive at **v3 at the earliest**, gated by a signer-isolation ADR.
- Watchlists/saved views exist pre-auth in a single `default` workspace; the schema carries `workspace_id` from day one so v2 is a data migration, not a redesign ([storage](../design/storage.md)).

## Consequences

- Key-security and PII surface at MVP is effectively zero; the [security-and-privacy](../ops/security-and-privacy.md) posture stays small and honest.
- The product can be useful and public immediately, without legal review of trading flows.
- Some users will ask for portfolio views early; the roadmap answers with v2, not scope creep.
- All execution-adjacent design in the vault is explicitly marked *future constraint*, never MVP work.

## Alternatives considered

- **Auth at v1** — rejected: pulls session management, PII, and deletion-rights work forward while the core research value is still unproven.
- **Execution as a launch goal** — rejected: custody-grade risk before product-market fit, and Polymarket's official CLI already covers basic order placement; owt would be differentiating on the wrong axis.

## Rollout notes

Revisit at v2 kickoff (SIWE ADR) and v3 kickoff (signer-isolation ADR). Any earlier account-data feature requires superseding this ADR.

## Related

- [../product/roadmap.md](../product/roadmap.md)
- [../ops/security-and-privacy.md](../ops/security-and-privacy.md)
- [../product/vision-and-scope.md](../product/vision-and-scope.md)
