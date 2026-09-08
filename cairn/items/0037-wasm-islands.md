---
id: 37
title: Wasm islands
type: feature
status: backlog
labels:
- out-of-scope
created: 2026-09-08
updated: 2026-09-08
priority: p3
effort: xl
area: ui
---

## Problem

Filed so that it stops being an open question. `doc/DESIGN` section 11 lists
this as deliberately excluded, and an exclusion that is not written down as
an item gets relitigated every few months.

## Proposal

Not now.

The case against, while the project is what it is: a landing page and a docs
site need no client state. Islands would mean a hydration story, a
component-boundary story, a bundling story and a wasm toolchain in the build
— and every one of those would warp decisions that are currently simple
because there is nothing to hydrate.

The case for, if it ever arrives: a genuine need for interactive
documentation — a live playground, a configuration builder — where the
alternative is hand-written JavaScript that the design system cannot type.

If that need arrives, the shape is probably Leptos or Dioxus components
mounted at marked elements, opted into per component, with the static render
remaining the fallback rather than the placeholder.

Reopen this when there is a specific page that needs it. "It would be nice"
is not that.

## Acceptance criteria

- [ ] Left closed until a concrete use case exists
- [ ] `doc/DESIGN` continues to say why
