---
id: 68
title: Content need not be a directory
type: feature
status: backlog
milestone: v1.2
depends_on:
- 67
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: content
---

## Problem

`Collection::load(dir, &directives, &context)` takes a path. Content must be
Markdown files on this machine.

That is the right default and a poor limit. The type is the schema — that is
the whole idea — and nothing about a type requires the bytes to have come from
a file. A changelog that lives in GitHub releases, a team page that lives in
a spreadsheet, a set of API docs generated from an OpenAPI document: each is
content, each has a shape, and each currently has to be checked into the
repository as Markdown by a script somebody maintains.

## Proposal

Astro's Content Layer is the right abstraction and its `Loader` is worth
reading closely: `{ name, load(context) }`, where the context supplies a
`store`, a `meta` KV, `parseData`, `renderMarkdown`, `generateDigest` and a
`watcher`.

The crabbucket version is smaller and better typed, because Astro's schema is
Zod — validated at runtime — and has to *generate* TypeScript types from it so
the rest of the program can see them. Here the Rust type is the schema and
there is nothing to generate.

A `Source` trait, roughly:

```rust sketch
pub trait Source {
    fn entries(&self, cx: &SourceContext<'_>) -> Result<Vec<Raw>>;
}
```

where `Raw` is a route, some frontmatter as a `toml::Table` or already-typed
value, a body, and a digest. `Collection<T>` then deserialises as it does now,
and every downstream gate — the layout check, the directive check, link
checking — applies unchanged, because they never knew where the bytes came
from.

Sources in the framework: a directory (what exists today, reimplemented on the
trait so the trait has a real user). Sources a site writes: whatever it likes.

Two hard constraints, both of which follow from decisions already made:

- **A build must not require the network.** A source that fetches has to be
  cacheable and the cache has to be committable, so `crab build` in CI and on
  an aeroplane both work. The digest and `meta` machinery from [[0067]] is
  what makes that possible, which is why this comes after it.
- **A source cannot be a runtime dependency.** Astro has grown "live"
  collections that fetch on request; that requires a server, and there is no
  server. A source runs at build time or not at all.

## Acceptance criteria

- [ ] A `Source` trait; the directory source implemented on it
- [ ] `Collection<T>` unchanged from a caller's point of view
- [ ] Every existing gate still applies to content from a non-file source
- [ ] A fetching source in `examples/`, with a committed cache, that builds
      offline
- [ ] `doc/DESIGN` states that a source is build-time only, and why
