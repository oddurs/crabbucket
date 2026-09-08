---
id: 8
title: Component style locality with crate-prefixed classes
type: feature
status: done
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

- [x] Each component's CSS lives beside the component
- [x] Class names carry a crate namespace applied in exactly one place
- [x] Two theme crates defining the same component name do not collide
- [x] The generated `site.css` is unchanged in effect for `examples/site`
- [x] `doc/DESIGN` section 10 matches what was built

## 2026-09-08

Done, but not as specified. The item and doc/DESIGN section 10 both called for a css! macro; there is no macro. A Style struct carrying (namespace, name, css) with '&' standing in for the class root does the whole job as a string replacement, and a proc-macro dependency would have bought nothing. DESIGN section 10 rewritten to describe what was built.

The stylesheet is now six files beside the components they style. Splitting it found a real regression: the landing-layout overrides were silently lost in the move, and only a check that every class in the built markup has a rule caught it. That check belongs in the integration suite (0002).

Cross-component styling is allowed within one design system -- site.css names cb-prose literally -- with a comment saying why: they ship in the same crate, so there is nothing to collide with, and a component that cannot style what it contains would be worse.

Theme::router_js now returns String rather than &str, because the theme's own class names go into the router and are not known until the theme is written.
