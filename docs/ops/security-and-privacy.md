---
type: ops
status: draft
owner: pinakin
summary: "Security model and privacy stance for the read-only v1, with forward constraints for the auth and execution phases."
tags: [area/security, release/mvp]
related:
  - ../adr/0008-read-only-through-v1.md
  - ../architecture/data-sources.md
  - ci-cd-and-release.md
---

# Security & privacy

The MVP's security posture is deliberately small because the product is deliberately small ([ADR-0008](../adr/0008-read-only-through-v1.md)): no wallets, no accounts, no PII, localhost by default. This note keeps that posture *honest* — and pins the constraints that later phases must design against.

## v1 trust model

| Property | Stance |
|---|---|
| Keys/custody | **None exist in the system.** No wallet code paths compile into MVP/v1 binaries. |
| PII | None collected. No user accounts; workspace data is anonymous local research state; TUI sends no telemetry. |
| Network exposure | `owtd` binds `127.0.0.1` by default. Remote self-host requires explicit `api.bind` + bearer token; TLS via reverse proxy ([deployment](deployment.md)). Admin port is localhost-only, always. |
| Data classification | Everything ingested is public-web or public-chain data; the only sensitive class is licensed news text (below). |

## Threat model (MVP, summary)

| Threat | Vector | Controls |
|---|---|---|
| Upstream data poisoning | a compromised/spoofed feed plants false timeline evidence | source authority ranking; `source_refs` provenance on every record; publisher allow-list for RSS registry; no auto-added feeds |
| SSRF via fetchers | feed URLs / canonical-URL resolution reaching internal networks | fetchers resolve through an allow-list of schemes (https), deny private IP ranges, cap redirects, no URL fetch from user input at MVP |
| Supply chain | malicious crate / action | `cargo deny` + audit, pinned actions by SHA, new-dep justification, SBOM ([ci-cd-and-release](ci-cd-and-release.md#supply-chain-ramp)) |
| Secrets leakage | `.env`, config, CI | secret scanning, `.env` gitignored with `.env.example` pattern, secrets only via env/secret manager, never in TOML committed to repo |
| Malicious API client | script hammering a self-host | server-side per-client rate limits, bearer token, bounded WS queues ([query-api](../design/query-api.md)) |
| Terminal escape injection | upstream text (news headlines, comments) containing ANSI/OSC sequences rendered in the TUI | **all upstream text is sanitized before render** (strip control sequences) — a research terminal renders adversarial text all day |

## Content compliance

- **News licensing:** default `license_class: metadata_excerpt` — headline, summary ≤ 2,000 chars, canonical URL; `body_text` stored only for feeds whose license class explicitly permits, with scheduled expiry where required ([storage](../design/storage.md#facts-append-only-hypertables-marked-)).
- **Robots & terms:** scrapers (last-resort tier) obey RFC 9309 robots.txt, per-source terms, and named-owner justification ([data-sources](../architecture/data-sources.md#compliant-scraping-last-resort-off-by-default)); X data (v2) stays within its ToS redistribution limits.
- **On-chain data:** public, but enrichment discipline applies — owt does not build persistent *person-level* behavioral profiles; holder/flow analytics (v1+) stay market-centric and address-pseudonymous.
- **Model outputs (v1):** owt is a research tool, not an advisor. Every forecast/odds value ships with `model_id`, `model_version`, its `signal`/`forecast` kind, and the disclosure *"Statistical model output for research purposes; not financial advice."* — rendered wherever the value appears, non-optional in clients ([ADR-0012](../adr/0012-forecast-derived-data-module.md)). Models are deterministic and statistical only (no LLM), so any published number is reproducible and auditable by a self-hoster.

## Privacy principles (binding from v2, designed-for now)

When accounts arrive (v2), these are the rules the auth ADR must satisfy — GDPR Art. 5/25 (minimization, protection by design/default) and CCPA deletion rights set the bar:

1. Collect the minimum: a SIWE address and chosen workspace data; no emails required, no tracking.
2. Deletion and export paths for workspace data ship *with* accounts, not after (test scenario 9 in [testing-strategy](testing-strategy.md#mandatory-scenarios)).
3. Public-chain data joined to an account becomes personal data — retention and access rules apply to the join, not just the raw.
4. TUI telemetry stays opt-in-never-default.

## Forward constraints (v2/v3 — recorded, not built)

- **Polymarket auth reality:** L1 = wallet-key EIP-712 signing; L2 = HMAC API credentials; order placement always requires user-side signing. Therefore owt will **never hold custody keys server-side**.
- **SIWE (EIP-4361)** for identity, **EIP-1271-aware** so contract wallets work; terminal flow is a device-flow analog (D-10).
- **v3 signer isolation:** signing happens in a separate local process (or hardware), talking to owt over an audited, allow-listed IPC; the main system remains read-only by construction. Requires its own ADR + threat model before any code.

## Control baseline (all phases)

secrets via env/manager only · encryption at rest on managed stores · immutable audit log for admin/config actions (from v2) · workspace-scoped authz checks on every query (schema ready now) · dependency review + SBOM every release · vulnerability reporting via root `SECURITY.md`.

## Related

- [../adr/0008-read-only-through-v1.md](../adr/0008-read-only-through-v1.md)
- [../architecture/data-sources.md](../architecture/data-sources.md)
- [ci-cd-and-release.md](ci-cd-and-release.md)
