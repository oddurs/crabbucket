---
id: 51
title: Frontmatter a site adds is silently dropped
type: bug
status: backlog
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

- [ ] A site can read frontmatter its design system defines
- [ ] A key nothing reads fails the build naming the file and the line
- [ ] The chosen shape is written down in doc/DESIGN with the reasoning
- [ ] `date` is reconsidered against it: it may belong in the site's own fields
