---
id: 8
title: Component style locality with crate-prefixed classes
type: feature
status: backlog
milestone: v0.1
labels:
- design
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: ui
---

## Problem

`doc/DESIGN` section 10, replacing the withdrawn tree-shaking promise.

A component's styles live in one big `BASE` constant at the bottom of
`crabbucket-ui`, far from the function they style. And nothing stops a second
theme crate from also defining `.card`, at which point whichever sheet loads
last wins and the bug is invisible until a page uses both.

## Proposal

Not a bundler. A naming mechanism plus a composition point.

1. A component declares its CSS next to itself, as an associated constant or
   a `const STYLE: &str`.
2. A `css!` macro prefixes class names with a crate-derived namespace, so
   `card` becomes `cb-card` in `crabbucket-ui` and `xy-card` elsewhere. The
   macro applies it in both the markup and the stylesheet, so the two cannot
   disagree.
3. `Theme::stylesheet` composes the sheet from the components the theme
   declares, deduplicated by name.

Explicitly out of scope: eliminating CSS for components that were not
rendered. That optimises the wrong number at this size, and the reasoning is
written down in `doc/DESIGN` so it does not get relitigated by accident.

## Acceptance criteria

- [ ] Each component's CSS lives beside the component
- [ ] Class names carry a crate namespace applied in exactly one place
- [ ] Two theme crates defining the same component name do not collide
- [ ] The generated `site.css` is unchanged in effect for `examples/site`
- [ ] `doc/DESIGN` section 10 matches what was built
