#!/usr/bin/env python3
"""Verification suite for the owt documentation vault."""
import os, re, sys, glob

ROOT = "/Users/pinakin/Projects/owt"
DOCS = os.path.join(ROOT, "docs")
failures, warnings = [], []

def rel(p): return os.path.relpath(p, ROOT)

md_files = sorted(glob.glob(os.path.join(DOCS, "**", "*.md"), recursive=True))
root_md = [os.path.join(ROOT, f) for f in ("README.md", "CONTRIBUTING.md", "SECURITY.md")]
all_md = md_files + root_md

# ---------- 1. Link integrity (relative links resolve) ----------
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
CODE_FENCE_RE = re.compile(r"```.*?```", re.S)
for f in all_md:
    text = open(f, encoding="utf-8").read()
    body = CODE_FENCE_RE.sub("", text)  # ignore links inside code fences
    body = re.sub(r"`[^`\n]*`", "", body)  # ignore inline code spans
    for target in LINK_RE.findall(body):
        if target.startswith(("http://", "https://", "mailto:")):
            continue
        path = target.split("#")[0]
        if not path:  # pure anchor
            continue
        resolved = os.path.normpath(os.path.join(os.path.dirname(f), path))
        if not os.path.exists(resolved):
            failures.append(f"BROKEN LINK  {rel(f)} -> {target}")

# ---------- 2. Frontmatter lint ----------
FM_RE = re.compile(r"\A---\n(.*?)\n---\n", re.S)
REQUIRED = ("type:", "status:", "summary:", "tags:")
TYPES = {"product","hld","lld","adr","ops","governance","index","research"}
STATUSES = {"draft","review","approved","superseded"}
summaries = {}
for f in md_files:
    text = open(f, encoding="utf-8").read()
    m = FM_RE.match(text)
    if not m:
        failures.append(f"NO FRONTMATTER  {rel(f)}")
        continue
    fm = m.group(1)
    for req in REQUIRED:
        if not re.search(rf"^{req}", fm, re.M):
            failures.append(f"FRONTMATTER MISSING {req:<9} {rel(f)}")
    t = re.search(r"^type:\s*(\S+)", fm, re.M)
    s = re.search(r"^status:\s*(\S+)", fm, re.M)
    if t and t.group(1) not in TYPES: failures.append(f"BAD type '{t.group(1)}'  {rel(f)}")
    if s and s.group(1) not in STATUSES: failures.append(f"BAD status '{s.group(1)}'  {rel(f)}")
    summ = re.search(r'^summary:\s*"?(.*?)"?\s*$', fm, re.M)
    if summ: summaries[rel(f)] = (summ.group(1), s.group(1) if s else "?")

# ---------- 3. MOC sync: index table rows match frontmatter ----------
index_text = open(os.path.join(DOCS, "README.md"), encoding="utf-8").read()
ROW_RE = re.compile(r"^\|\s*\[([^\]]+)\]\(([^)]+)\)\s*\|\s*(\w+)\s*\|\s*(.+?)\s*\|$", re.M)
for label, path, status, summary in ROW_RE.findall(index_text):
    key = os.path.join("docs", path)
    if key not in summaries:
        continue  # ADR table rows have no summary column; skipped by regex shape anyway
    fm_summary, fm_status = summaries[key]
    if summary.strip() != fm_summary.strip():
        failures.append(f"MOC SUMMARY DRIFT  {key}\n    index: {summary.strip()}\n    front: {fm_summary.strip()}")
    if status.strip() != fm_status.strip():
        failures.append(f"MOC STATUS DRIFT  {key}: index={status} front={fm_status}")

# ---------- 4. Banned terms ----------
BANNED = re.compile(r"Fable|F#|\bElmish\b|\bxterm\b|\.NET\b|ASP\.NET|\bInk\b|Playwright|SAFE-stack")
SANCTIONED = {"docs/adr/0001-rust-backend.md", "docs/adr/0002-ratatui-tui-first-client.md",
              "docs/research/plan.md", "docs/ops/testing-strategy.md",
              "docs/product/roadmap.md"}  # roadmap divergences + testing supersession notes
for f in all_md:
    if rel(f) in SANCTIONED: continue
    for i, line in enumerate(open(f, encoding="utf-8"), 1):
        if BANNED.search(line):
            failures.append(f"BANNED TERM  {rel(f)}:{i}  {line.strip()[:90]}")

# ---------- 5. citation artifacts (plain or PUA-wrapped) + stray PUA chars ----------
ARTIFACT_RE = re.compile(r"cite[-]?turn")
PUA_RE = re.compile(r"[-]")
for f in all_md:
    if rel(f) == "docs/research/plan.md": continue
    text = open(f, encoding="utf-8").read()
    n = len(ARTIFACT_RE.findall(text))
    if n: failures.append(f"CITETURN ARTIFACT  {rel(f)} x{n}")
    p = len(PUA_RE.findall(text))
    if p: failures.append(f"PRIVATE-USE CHARS  {rel(f)} x{p}")
plan_md = os.path.join(DOCS, "research/plan.md")
report_citeturns = len(ARTIFACT_RE.findall(open(plan_md, encoding="utf-8").read())) if os.path.exists(plan_md) else 0

# ---------- 6. wikilinks (outside inline code) ----------
for f in all_md:
    body = CODE_FENCE_RE.sub("", open(f, encoding="utf-8").read())
    body = re.sub(r"`[^`]*`", "", body)  # strip inline code spans
    if "[[" in body:
        failures.append(f"WIKILINK  {rel(f)}")

# ---------- 7. Mermaid sanity ----------
MTYPES = ("flowchart", "sequenceDiagram", "graph", "erDiagram", "stateDiagram")
mermaid_count = 0
for f in all_md:
    text = open(f, encoding="utf-8").read()
    for block in re.findall(r"```mermaid\n(.*?)```", text, re.S):
        mermaid_count += 1
        first = block.strip().splitlines()[0].strip()
        if not first.startswith(MTYPES):
            failures.append(f"MERMAID TYPE?  {rel(f)}: starts '{first[:40]}'")

# ---------- 8. Size discipline ----------
for f in md_files:
    n = sum(1 for _ in open(f, encoding="utf-8"))
    if rel(f) == "docs/research/plan.md": continue
    if n > 500: failures.append(f"SIZE >500  {rel(f)}: {n} lines")
    elif n > 300: warnings.append(f"size 300-500  {rel(f)}: {n} lines")

# ---------- report ----------
print(f"files checked: {len(all_md)} ({len(md_files)} vault + {len(root_md)} root)")
print(f"mermaid blocks: {mermaid_count}; citeturn in research/plan.md: {report_citeturns}")
print(f"\nWARNINGS ({len(warnings)}):")
for w in warnings: print("  " + w)
print(f"\nFAILURES ({len(failures)}):")
for x in failures: print("  " + x)
sys.exit(1 if failures else 0)
