---
id: 67
title: Rebuild only what changed
type: feature
status: backlog
milestone: v1.2
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: build
---

## Problem

Every build reads, parses, renders and writes every page. At 521 pages that is
250ms and nobody minds. At five thousand it is two and a half seconds, and the
dev loop — which rebuilds on every keystroke pause — stutters.

And it is the wrong 250ms. [[0036]] measured where it goes: 79% is read and
parse, dominated by highlighting roughly 2,000 code blocks. Editing one
paragraph re-highlights all of them.

## Proposal

Astro's Content Layer has the recipe, and its `LoaderContext` names the two
pieces: a `store` for the data and a `meta` — "a simple KV store, designed for
things like sync tokens" — plus `generateDigest`, "a non-cryptographic content
digest. This can be used to check if the data has changed".

For crabbucket:

- **A digest per source file**, kept in `.crabbucket/` beside the card cache.
- **A cache of the expensive intermediate**, which is the rendered HTML and
  heading list for a page — the output of the read-and-parse pass, keyed by
  (digest, crabbucket version, theme version). That is the 79%.
- **Invalidate correctly, which is the whole difficulty.** A page's own bytes
  changing invalidates that page. But `site.toml` changing invalidates
  everything; a data file changing invalidates every page whose directives
  read it; and the route set changing invalidates *link checking for every
  page*, because a link is dead only relative to the finished set of routes.

That last one is the interesting constraint and it comes straight out of the
existing architecture. Rendering can be incremental. **Link checking cannot
be, and must not pretend to be** — it is 2ms for 3,500 links, so it should
simply always run in full. An incremental link checker would be a fast way to
stop noticing dead links, which is the one thing this project may not do.

So the shape is: incremental read-and-parse, incremental render, always-full
check, incremental write. Which is also the ordering the passes already have.

Correctness requirements, because a stale cache that produces a wrong site is
worse than a slow build:

- A cold build and a warm build must produce byte-identical output. The
  two-directory determinism test from [[0055]] extends to cover this: build,
  edit one page, rebuild, and compare against a cold build of the same input.
- `crab build --no-cache` exists and is what CI uses, so the published site is
  never a cache artefact.
- The cache is versioned, and a version mismatch discards it rather than
  attempting migration.

## Acceptance criteria

- [ ] Per-file digests and a cached parse result in `.crabbucket/`
- [ ] Editing one page re-parses one page
- [ ] `site.toml` and data files invalidate what they should, with tests
- [ ] Link checking always runs in full, and a test asserts it
- [ ] Warm output is byte-identical to cold output, asserted
- [ ] `--no-cache`, used by CI
- [ ] Measured: warm rebuild time for a one-page edit at 521 and 5,000 pages
