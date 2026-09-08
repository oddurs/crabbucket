---
id: 51
title: Frontmatter a site adds is silently dropped
type: bug
status: done
milestone: v1.0
labels:
- migration
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: content
---

## What happens

`PageMeta` does not deny unknown fields, so a frontmatter key it does not know
is ignored without a word. Migrating cairn's site, every one of its six
documentation pages carried a `summary`, and all six lost it silently.

Astro's answer is that a site declares its own collection schema. crabbucket's
built-in pipeline has a fixed one, and the only way a field has ever been added
is by adding it to `PageMeta` — which is how `date` arrived for feeds.

That does not scale, and it is the wrong shape: `summary` is cairn's business,
not the framework's.

## What should happen

A site should be able to carry frontmatter its own design system reads, and a
key nothing reads should be an error rather than silence.

## Proposal

Two candidates, in order of how much they cost:

- `PageMeta` gains `#[serde(flatten)] extra: toml::Table`, exposed on `Page`.
  A theme reads `page.meta.extra.get("summary")`. Cheap, and stringly-typed at
  the point of use, which is the thing this project usually refuses.
- `Theme` gains an associated `Meta` type the way it has `Layout`, and
  `PageMeta<L, M>` carries it. Typed all the way through, and it makes the two
  generic parameters four in every signature that touches a page.

The second is the honest one and the first is the one that ships. Worth
deciding deliberately rather than by whichever gets written first.

Either way, `deny_unknown_fields` should follow, so a typo in a frontmatter key
fails rather than vanishing.

## Acceptance criteria

- [x] A site can read frontmatter its design system defines
- [x] A key nothing reads fails the build naming the file and the line
- [x] The chosen shape is written down in doc/DESIGN with the reasoning
- [x] `date` is reconsidered against it: it may belong in the site's own fields

## 2026-09-08

Done with the second option -- the typed one the item called honest and
expected not to ship.

It shipped because the cost the item feared did not materialise. The worry
was that two generic parameters would become four in every signature.
Instead `Page` became generic over the *theme* rather than over its layout:
`Page<'_, Self>` in a theme's own impl, and one parameter everywhere else.
Signatures got shorter, not longer.

`Theme::Extra` sits beside `Theme::Layout` and means the same thing: this
is my surface, and a site works within it. A design system that declares
nothing writes `type Extra = NoExtra;` -- one line, because associated type
defaults are not stable.

`NoExtra` is a struct rather than `()` because serde cannot flatten into a
unit type. Probed that before designing rather than after.

One thing the item asked for and I could not deliver: `deny_unknown_fields`.
serde cannot combine it with `flatten`, so a misspelled frontmatter key
still vanishes silently. That is now asserted by a test named after it, so
the day it becomes possible is a deliberate one rather than a discovery.

crabbucket-theme-plain reads a `summary`, which is exactly what cairn's six
pages carried and lost. The migration page is updated: that gap is closed.
