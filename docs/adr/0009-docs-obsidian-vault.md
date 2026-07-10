---
type: adr
status: approved
owner: pinakin
summary: "Documentation lives in an Obsidian-compatible vault of atomic notes with standard Markdown links."
tags: [area/product]
related:
  - ../README.md
  - README.md
---

# ADR-0009: Documentation as an Obsidian-compatible vault

## Context

The founding documentation suite (~30 notes) needs to serve three readers at once: humans browsing GitHub, humans exploring relationships in Obsidian's graph view, and AI agents that should load the *smallest* set of notes sufficient for a task instead of a monolithic architecture document. The founder explicitly asked for Obsidian-based documentation to get the relation graph and to optimize token usage during agent-assisted development.

## Decision

- Documentation lives in-repo at `docs/`, which **is** the Obsidian vault root (code, fixtures, and build output stay outside the vault and therefore outside the graph).
- Notes are **atomic**: one topic per file, target 100–300 lines, soft cap 500.
- Every note carries YAML frontmatter — `type`, `status`, `owner`, `summary` (mandatory, mirrored verbatim in the [vault index](../README.md)), `tags` (controlled vocabulary), `related` (repo-relative paths).
- Links are **standard relative Markdown links**; `[[wikilinks]]` are banned. A minimal `docs/.obsidian/app.json` (`useMarkdownLinks: true`, `newLinkFormat: relative`) is committed so Obsidian generates conforming links; the rest of `.obsidian/` is gitignored.
- Each note ends with a `## Related` section repeating the frontmatter links as body links, which is what feeds Obsidian's graph and backlinks.
- Diagrams are Mermaid only.

## Consequences

- GitHub, Obsidian, and plain editors all render every note and resolve every link; a future static-site generator (mdBook/MkDocs) can consume the tree unchanged.
- An agent can answer most questions by reading `docs/README.md` plus one note — the `summary` field is the routing key. This is the token-usage optimization made concrete.
- CI must enforce the conventions (link integrity, frontmatter validity, index/summary sync, banned `[[wikilink]]` grep) — specified in [ci-cd-and-release](../ops/ci-cd-and-release.md).
- Contributors editing outside Obsidian must write relative paths by hand; the link checker catches mistakes.
- Notes must be split when they outgrow the size cap, which costs an occasional link-preserving refactor.

## Alternatives considered

- **`[[Wikilinks]]`** — Obsidian's default; rejected because GitHub renders them as literal bracket text and no path is carried, so agents and link checkers cannot resolve them without vault-wide name resolution.
- **Repo root as vault** — rejected: `src/`, fixtures, and generated files would pollute the graph and Obsidian search.
- **A few monolithic documents** (single ARCHITECTURE.md) — rejected: forces every reader and agent to load everything; defeats the graph.
- **Docs site generator now** (Docusaurus/MkDocs) — deferred: adds build tooling before there is an audience; the vault remains compatible if wanted later.
- **Notion workspace** — rejected: detached from the repo, not PR-reviewable, hostile to OSS contributors.

## Rollout notes

Adopted at vault creation (2026-07-07), before any other note was written, so no migration was needed. The original research report was moved verbatim into the vault as `research/plan.md` with provenance frontmatter, then later removed outright (2026-07-07) once its stack-specific recommendations were fully superseded — see [ADR-0001](0001-rust-backend.md) and [ADR-0002](0002-ratatui-tui-first-client.md) for the surviving record of what it recommended and why it was rejected.

## Related

- [../README.md](../README.md) — conventions this ADR anchors
- [README.md](README.md) — ADR process
