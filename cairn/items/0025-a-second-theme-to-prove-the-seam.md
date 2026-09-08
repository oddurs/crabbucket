---
id: 25
title: A second theme, to prove the seam
type: feature
status: backlog
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: theme
---

## Problem

`Theme` has exactly one implementation. A trait with one implementor is a
guess about what the abstraction needs, and it is usually wrong in ways that
only the second implementor reveals.

Given that swapping design systems is a headline claim, shipping it untested
is the kind of thing that is embarrassing later.

## Proposal

`crabbucket-theme-plain`: a deliberately different design system, not a
restyle. Serif body text, a single column, no sidebar, light by default, one
layout instead of three, no client router.

That last constraint is the load-bearing one. A theme with one layout and no
router will expose every place the current design assumes there are three
layouts and a `router.js` to serve — which is the entire point of writing it.

Build `examples/site` with both, in CI, and diff nothing: the assertion is
simply that both succeed and that no change to `crabbucket` was needed to
make the second one work.

Expect this to change the `Theme` trait. Record what changed and why on this
item; that record is the actual deliverable.

## Acceptance criteria

- [ ] A second theme crate with a genuinely different structure
- [ ] `examples/site` builds under both, in CI
- [ ] Any trait changes it forced are documented on this item
- [ ] A theme may decline the router without special-casing in the framework
- [ ] The docs page on themes is written against two real examples
