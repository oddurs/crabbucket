---
id: 28
title: Generated OpenGraph images
type: feature
status: backlog
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: l
area: build
---

## Problem

A link to a docs page shared anywhere renders as a grey rectangle. For a
project whose pitch is that things should look considered, that is the most
visible unconsidered surface it has.

## Proposal

Render a card per page at build time: the page title, the site title, and the
design system's own colours, so the image is themed by the same token file as
everything else.

Approach, cheapest first:

1. Compose an SVG from tokens and rasterise with `resvg`. No browser, no
   headless anything, and text layout is the only hard part.
2. Draw directly with `tiny-skia` and a bundled font, if SVG text metrics
   prove fiddly.

Font licensing matters: bundle something with a licence that permits
redistribution, and record which and why.

Cache aggressively — key on title plus token hash — because regenerating
forty images on every build would undo the fast-build property this project
otherwise has.

Emit `og:image` and `twitter:card` only when `url` is set, since both need
absolute URLs.

## Acceptance criteria

- [ ] One image per page, generated at build time
- [ ] Colours and fonts come from the token file
- [ ] Cached on title and token hash
- [ ] Meta tags emitted only when `url` is configured
- [ ] Font licence recorded
- [ ] Build time impact measured and recorded
