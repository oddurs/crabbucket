---
id: 2
title: Test the build gates end to end
type: chore
status: done
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

- [x] Every `Error` variant has a fixture that produces it
- [x] Each failing fixture asserts the message names the offending file
- [x] `base-path/` asserts no double prefix and no missing prefix
- [x] Fixtures live under `crates/crabbucket/tests/sites/`
- [x] `make check` runs them

## 2026-09-08

Done. Twelve fixture sites under crates/crabbucket/tests/sites, seventeen tests in tests/build.rs, plus four in crabbucket-ui/tests/stylesheet.rs.

The test theme is defined in the test file rather than borrowed from crabbucket-ui: the dependency would run the wrong way, and it keeps Theme honest -- if the trait cannot be implemented from outside in thirty lines, that is worth finding out here. It took twenty.

Output goes to a per-call temp directory, not per-fixture, because two tests build the same fixture and the suite runs them at once. The first version collided and failed intermittently.

One assertion had to be softened after being wrong rather than the code being wrong: overriding --base does not rewrite a site-absolute link hard-coded in content, and should not, because such a link may belong to something else served from the same domain. The test now asserts that explicitly, with the reasoning, so nobody 'fixes' it later.
