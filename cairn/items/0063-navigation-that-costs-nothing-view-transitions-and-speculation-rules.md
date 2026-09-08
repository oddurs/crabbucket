---
id: 63
title: 'Navigation that costs nothing: view transitions and speculation rules'
type: feature
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: ui
---

## Problem

The router is 2.9KB gzipped and exists to do two things the platform now does
natively: prefetch on intent, and navigate without a white flash.

`@view-transition { navigation: auto; }` is one line of CSS and gives
cross-document transitions. A speculation rules script tag gives prefetch and
prerender with no JavaScript at all. Both are ignored silently by browsers
that do not implement them, which makes them free.

## Proposal

Ship both as progressive enhancement, and make the router optional rather
than the only answer.

- `@view-transition { navigation: auto; }` in the default stylesheet, plus
  `view-transition-name` on the masthead and the main region so the shell
  stays put.
- Speculation rules as an opt-in alternative to the router's prefetching: a
  `<script type="speculationrules">` block with a conservative
  `eagerness: "moderate"` document rule scoped to same-origin links.
- A site can then choose: nothing, speculation rules and view transitions
  (zero bytes), or the router (2.9KB, and works everywhere today).

**What this item does not do is delete the router**, and the reason is worth
writing down rather than assuming. As of now cross-document view transitions
are in Chrome and Edge 126+ and Safari 18.2+ but not Firefox, so they are
explicitly not Baseline; speculation rules are Chromium-only. Both are Interop
2026 focus areas.

So the retirement condition goes in `doc/DESIGN` as a dated, checkable claim:
when cross-document view transitions reach Baseline and the platform covers
prefetching, the router becomes 2.9KB that buys nothing and should be deleted.
Until then it is what makes the behaviour uniform.

The honest secondary benefit: the router's `crabbucket:render` event and its
focus and announcement handling exist because a client-side navigation breaks
both. Cross-document navigation does not break either — the platform handles
focus and screen-reader announcement itself. A site on the zero-byte path gets
*better* accessibility than one on the router, for free, which is a good
argument and slightly embarrassing.

## Acceptance criteria

- [ ] `@view-transition` and `view-transition-name` in the default stylesheet
- [ ] Speculation rules available, off by default, same-origin only
- [ ] A site can take the zero-byte path and lose nothing but old-browser
      smoothness
- [ ] The router still passes its existing focus and announcement tests
- [ ] `doc/DESIGN` records the dated retirement condition for the router
