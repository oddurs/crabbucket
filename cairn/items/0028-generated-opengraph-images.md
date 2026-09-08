---
id: 28
title: Generated OpenGraph images
type: feature
status: done
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

- [x] One image per page, generated at build time
- [x] Colours and fonts come from the token file
- [x] Cached on title and token hash
- [x] Meta tags emitted only when `url` is configured
- [x] Font licence recorded
- [x] Build time impact measured and recorded

## 2026-09-08

Done with the first of the two approaches the item listed: an SVG composed
from tokens, rasterised with resvg.

**Cost, measured.** 76 transitive crates and a few seconds of compilation.
That is why it is `crabbucket-og` and not part of `crabbucket`: a docs site
with no interest in social previews should pay neither. A design system that
wants cards depends on it and implements `Theme::og_image`.

**The cache is not optional.** A card is about 65ms; fourteen pages is most
of a second on a build that otherwise takes a tenth of one. Cold 0.37s, warm
0.12s. It lives in `.crabbucket/` beside the site rather than in `dist/`,
which is deleted every build, and the key covers the words, the colours, the
size and a hand-bumped layout version.

**Font licensing.** Public Sans, SIL OFL, recorded in `fonts/README` and
`fonts/OFL.txt`. Two static instances rather than the variable family:
resvg does not apply a variable font's weight axis, so the variable file
rendered every title at Thin whatever `font-weight` said. The first card I
looked at was visibly wrong, which is the argument for looking.

**A finding.** The first draft of the docs page embedded the card it
generates. Building the example site with crabbucket-theme-plain failed on a
dead link, correctly: content that references a generated asset is coupled to
a design system that generates it. The page now embeds a committed copy and
says why. That is the third time the second design system has caught
something no single-theme test could.

Not built, and worth saying: no per-page artwork, no description text, and
line breaking estimates width from the character count rather than measuring
it. A card is one short line of display type in one known font; measuring
properly would mean shaping the text twice.
