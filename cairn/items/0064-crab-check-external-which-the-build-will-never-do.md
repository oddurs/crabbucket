---
id: 64
title: crab check --external, which the build will never do
type: feature
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: build
---

## Problem

External links are not checked at all. A link to a page that moved three years
ago is indistinguishable from a good one.

The build must not check them. That position is already argued in
`doc/routing` — "a generator that phones out to the network to decide whether
it succeeded is a generator that fails on an aeroplane" — and it is right. A
build that depends on the reachability of somebody else's server is a build
that is not reproducible and not offline.

But refusing to do it *during the build* is not a reason never to do it.

## Proposal

A separate command, which is never part of `build` and never part of `make
check`:

```
crab check --external [DIRECTORY]
```

It builds the site into a temporary directory, collects every external link,
and reports. Zola's checker is the reference implementation and has the pieces
worth copying:

- A shared cache keyed by URL, so a link repeated across forty pages is one
  request. Zola holds it in a `LazyLock<Arc<RwLock<HashMap<String, Result>>>>`.
- One reusable client, for connection pooling.
- Anchor checking: fetch the body and confirm the fragment exists. Zola has a
  `skip_anchor_prefixes` config for sites where that is hopeless, and skips
  anchors that look like client-side routes (`#/`, `#!`).
- A `HEAD` first, falling back to `GET`, since many servers reject `HEAD`.

Two things Zola does not do that this should, because they are the difference
between a useful report and an annoying one:

- **Persist the cache between runs**, with a TTL, in `.crabbucket/`. A link
  that was alive this morning does not need re-checking this afternoon.
- **Distinguish gone from unreachable.** A 404 is the site's problem; a
  timeout, a 429 or a TLS error is the network's, and reporting them
  identically is how a checker gets ignored. Exit non-zero only for the
  first class; report the second and keep the status.

Configuration for links that are permanently unverifiable — anything behind a
login, anything that blocks robots — belongs in `site.toml` with a reason
beside each, in the same spirit as the `deny.toml` advisory ignores.

## Acceptance criteria

- [ ] `crab check --external` exists and `crab build` still never uses the
      network
- [ ] Cached in `.crabbucket/` with a TTL, and deduplicated within a run
- [ ] Fragments checked, with a way to skip them per prefix
- [ ] Gone and unreachable reported separately, and only gone fails
- [ ] Not in `make check`; a scheduled CI job instead, like the fuzzers
