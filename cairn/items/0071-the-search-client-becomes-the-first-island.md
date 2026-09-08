---
id: 71
title: The search client becomes the first island
type: feature
status: backlog
milestone: v2.0
depends_on:
- 37
- 70
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: ui
---

## Problem

[[0037]] and [[0070]] are a trait and a toolchain. A trait with no
implementation is a guess about what the abstraction needs — the project
learned that from writing a second design system, and the lesson cost three
changes to the framework.

So islands need a first real user, chosen because it is genuinely the right
shape rather than because it is convenient.

## Proposal

The search box, for four reasons that all point the same way:

- **It is the untyped seam.** Hand-written JavaScript agreeing a JSON shape
  with a Rust function by hand. Making it an island is not a demonstration; it
  is the fix.
- **It shares a type with the generator.** After [[0060]] the index is a set of
  Rust structs. The island deserialises exactly those, so the format cannot
  drift from the reader.
- **It is the hardest case, which is why it is the right one.** It fetches,
  it holds state, it renders a list, it handles keyboard navigation and focus,
  and it must be accessible. If the trait can carry search it can carry a
  playground.
- **It has a fallback that is genuinely useful**, not a spinner: the static
  render is a form that submits to a `/search/` page rendered at build time
  from the same index. A reader with no JavaScript searches the site.

This item is also where the claims get measured rather than asserted: bytes
over the wire against the current JavaScript client, and time to first result.
If the typed version is bigger and slower, that goes on the Speed page, in the
same words the Hugo comparison got.

## Acceptance criteria

- [ ] Search is an island; the index types are shared with the generator
- [ ] A change to the index format that breaks the client is a compile error
- [ ] With JavaScript off, search works via a build-time `/search/` page
- [ ] Keyboard navigation, focus handling and announcement tested
- [ ] Bytes and time-to-first-result measured against the current client, and
      published whichever way they come out
- [ ] The old hand-written client deleted, not left beside it
