---
id: 25
title: A second theme, to prove the seam
type: feature
status: done
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

- [x] A second theme crate with a genuinely different structure
- [x] `examples/site` builds under both, in CI
- [x] Any trait changes it forced are documented on this item
- [x] A theme may decline the router without special-casing in the framework
- [x] The docs page on themes is written against two real examples

## 2026-09-08

Done. `crabbucket-theme-plain`: light, serif, one column, one layout, no
JavaScript at all. It renders crabbucket's own documentation site unmodified.

Five things changed in crabbucket because of it, and those are the
deliverable rather than the theme:

1. `Theme::router_js` and `search_js` return `Option<String>`. They returned
   `String` and `&str`, so a design system without a router had no way to say
   so and a site asking for one got an empty file. A site that asks now gets
   a warning and a page that works.

2. The token generator became `crabbucket-tokens`. It lived in
   `crabbucket-ui`'s build script, so Plain's first draft was a forty-line
   copy of it. The new crate's only dependency is `toml`, which also answers
   the crate-weight worry filed against 0024.

3. That generator learned which scheme a palette *is*. It assumed a dark base
   with light overrides, because that was the only palette it had ever seen.
   Plain is light-first.

4. The contrast maths moved there too, for the same reason: the second theme
   needed the same forty lines as the first.

5. `Layout` and directives turned out to be the portability contract, which
   the first design system could not reveal. A page's `layout` and its
   `:::directives` name things in a design system, so a theme meant to be
   swapped in has to answer to the names the content already uses. Plain does:
   two `serde(alias)` attributes and one module of deliberately plain
   components -- a callout is an indented note, cards are a list, tabs are
   every panel one after another, because a design system with no scripting
   cannot hide one behind another and pretending otherwise would leave half
   the page unreadable.

Nothing else in `crabbucket` needed changing to make Plain render, and no
crabbucket-ui class appears in Plain's output. That part went right.

One gap found and not fixed here: content has no way to write a site-absolute
link that survives the base path changing. The example site's 404 page
hard-codes /crabbucket/docs/, so building it with --base fails. Filed
separately.
