---
id: 2
title: Test the build gates end to end
type: chore
status: backlog
milestone: v0.1
labels:
- correctness
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: build
---

## Problem

The gates *are* the product. "The build either fails, or the site is correct"
is a claim about failure, and every claim about failure that is not tested is
a claim that will quietly stop being true.

Today the only end-to-end coverage is that `examples/site` happens to build in
CI. Nothing asserts that a bad site *fails*, or that it fails with a message
naming the file.

## Proposal

A `tests/` directory in `crabbucket` holding fixture sites, each a few files,
each aimed at exactly one gate. Drive them through `crabbucket::build` with a
test theme, and assert on the `Error` variant and on the rendered message.

Fixtures to start with:

- `ok/` — builds, asserts route set, asserts `.nojekyll` exists
- `missing-title/` — `Error::Frontmatter`, message names the file
- `no-frontmatter/` — `Error::MissingFrontmatter`
- `unterminated/` — `Error::UnterminatedFrontmatter`
- `unknown-layout/` — `Error::Frontmatter`, message lists the valid layouts
- `dead-route/` — `Error::DeadLinks`, one entry, correct `target`
- `dead-asset/` — `Error::DeadLinks` for a missing image
- `base-path/` — built with `base = "/repo/"`, asserts every emitted href is
  prefixed exactly once
- `drafts/` — a draft is skipped and is *not* linkable from a live page

Use a temporary output directory rather than writing into the fixture, so the
suite is safe to run in parallel and leaves no untracked files.

## Acceptance criteria

- [ ] Every `Error` variant has a fixture that produces it
- [ ] Each failing fixture asserts the message names the offending file
- [ ] `base-path/` asserts no double prefix and no missing prefix
- [ ] Fixtures live under `crates/crabbucket/tests/sites/`
- [ ] `make check` runs them
