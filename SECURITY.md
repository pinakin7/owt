# Security policy

## Supported versions

owt is pre-release; only the latest tagged release (and `main`) receive security fixes.

## Reporting a vulnerability

Please report vulnerabilities **privately** via GitHub Security Advisories ("Report a vulnerability" on the repository's Security tab). Do not open public issues for security reports, and do not include exploit details in PR descriptions.

You can expect: acknowledgment within 72 hours, a triage verdict within 7 days, and coordinated disclosure — we ask for up to 90 days before public disclosure, shorter by mutual agreement once a fix ships.

## Scope notes

- owt is **read-only through v1**: it holds no wallets, no private keys, and no user PII ([design rationale](docs/adr/0008-read-only-through-v1.md)). Reports about custody are out of scope until the v3 execution-adjacent phase exists.
- In scope and especially valued: ingestion pipeline abuse (SSRF via feeds, data poisoning), terminal escape-sequence injection through rendered upstream text, API auth bypasses on remote self-hosts, and supply-chain issues in the dependency tree.
- Self-hosts are the deployment model; issues requiring a hostile local user on the same machine are generally out of scope.

## Secret handling rules

No secrets in the repository, ever — configuration secrets come from environment variables or a secret manager (`.env` is gitignored; `.env.example` documents shape only). Secret scanning runs in CI; a leaked secret is treated as compromised and rotated, not deleted from history quietly.
