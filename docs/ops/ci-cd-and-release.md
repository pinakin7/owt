---
type: ops
status: draft
owner: pinakin
summary: "CI/CD workflow set, release artifacts and distribution, versioning and skew policy, and the supply-chain ramp."
tags: [area/ops, area/release, release/mvp]
related:
  - testing-strategy.md
  - deployment.md
  - ../design/tui-client.md
---

# CI/CD & release

GitHub Actions, release-train discipline ([roadmap](../product/roadmap.md)). Workflows are specified here and land as `.github/workflows/*.yml` in the first code phase — this doc is their contract.

## Workflow set

| Workflow | Trigger | Does |
|---|---|---|
| `ci.yml` | PR, main | fmt check → clippy `-D warnings` → build → `cargo nextest` (unit/property/contract) → integration (services via containers) → API contract + OpenAPI additive diff → TUI golden frames → **docs job** |
| `docs job` (in ci) | PR touching `docs/` | lychee link check (offline mode) → frontmatter validator (schema + index-summary sync) → banned-term grep → `[[wikilink]]` grep → mermaid parse |
| `drift-canary.yml` | daily cron | contract tests against **live** upstream endpoints; failure opens a labeled issue ([testing-strategy](testing-strategy.md#fixture-policy)) |
| `e2e.yml` | merge queue | PTY smoke suite (expectrl demo-script run) |
| `perf.yml` *(v1 gate)* | pre-release | k6 vs staging; fails on SLO regression ([query-api budgets](../design/query-api.md#latency-budgets-server-side-warm-k6-enforced-from-v1)) |
| `security.yml` | PR + weekly | `cargo audit`, `cargo deny` (licenses + bans), GitHub dependency review, secret scanning, SBOM (CycloneDX) attached to releases |
| `release.yml` | tag `v*` | build GHCR `owtd` image (multi-arch) + cargo-dist `owt` matrix → **draft** GitHub release with binaries, SBOM, checksums, release notes from template → publish on manual approval |
| `deploy.yml` *(v1)* | release published | staging promote → migration step ([deployment](deployment.md#migrations-flow)) → smoke → manual prod gate; rollback = previous image redeploy |

## Release artifacts

| Artifact | Channel |
|---|---|
| `owtd` container image (linux/amd64, linux/arm64) | GHCR, tagged `vX.Y.Z` + `latest` on stable |
| `owt` binaries: macOS x86_64/aarch64, Linux x86_64/aarch64 (musl static), Windows x86_64 | GitHub Release assets via cargo-dist; checksums + (v3) signatures |
| `docker-compose.yml` + `.env.example` | in-repo, referenced by release notes |
| SBOM | release asset from `security.yml` |
| Homebrew tap | once release cadence stabilizes (post-MVP) |

## Versioning & skew

- One version for the workspace (`vX.Y.Z`), tagged on the train.
- **MVP: lockstep** — `owt` and `owtd` must match minor; the WS `hello`/`welcome` handshake enforces it (coded close `UPGRADE_REQUIRED`).
- **From v1: semver compatibility** — client works against any server with the same major and server minor ≥ client minor; `/v1` REST additive-only rule makes this real ([query-api § versioning](../design/query-api.md#versioning--auth-posture)).
- Pre-1.0 caveat documented in release notes: minors may break, patches never.

## Release notes template

```markdown
# owt vX.Y.Z — <train name>
## Highlights
## Breaking changes        <!-- pre-1.0 only; empty is the goal -->
## Migration steps         <!-- owtd migrate notes, config changes -->
## New commands / keys
## Fixed issues
## Known limitations
## Verification checklist  <!-- demo-script result, SLO snapshot -->
```

## Changelog

A root [CHANGELOG.md](../../CHANGELOG.md) tracks notable changes in
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) form under a running
`[Unreleased]` heading (grouped `Added` / `Changed` / `Deprecated` / `Removed` /
`Fixed` / `Security`).

- Update `[Unreleased]` in the **same PR** as any user-facing change — a new command or
  key, an API/DTO change, a fixed behavior, or a breaking change.
- At release, `release.yml` cuts `[Unreleased]` into a dated `vX.Y.Z` section and starts
  a fresh one; the cut entries are the source material for the GitHub release notes
  rendered from the template above.
- Pre-1.0, a `Removed`/`Changed` entry that breaks compatibility maps to the notes'
  **Breaking changes**, and any migration-affecting entry maps to **Migration steps**.

## Supply-chain ramp

- **MVP:** pinned GitHub Actions (by SHA), `cargo deny` license/ban policy, dependency-review gate, secret scanning, SBOM per release, new-dependency justification rule ([cargo-workspace](../architecture/cargo-workspace.md#workspace-policy)).
- **v2:** provenance attestations on images.
- **v3 (execution-adjacent):** signed artifacts (Sigstore), reproducible-build check, mandatory review on signer-path crates.

## Related

- [testing-strategy.md](testing-strategy.md)
- [deployment.md](deployment.md)
- [../design/tui-client.md](../design/tui-client.md)
