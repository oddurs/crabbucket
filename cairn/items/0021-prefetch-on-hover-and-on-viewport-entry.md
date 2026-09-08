---
id: 21
title: Prefetch on hover and on viewport entry
type: feature
status: backlog
milestone: v0.2
depends_on:
- 17
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: router
---

## Problem

The router fetches on click, so a navigation still waits a round trip. For a
site this small the whole page is smaller than the round trip that fetches
it, which makes the wait pure overhead.

## Proposal

Prefetch a route's HTML when the pointer rests on its link, and when a link
enters the viewport on a fast connection. Both are cheap; both need limits.

- Respect `navigator.connection.saveData` and `effectiveType`.
- Cap the cache — a handful of pages, evicted oldest first.
- Never prefetch on `pointerdown`; by then the fetch is happening anyway.
- Hold results in memory only. A cache that outlives the page is a cache that
  serves stale content after a deploy.

Measure it before keeping it. If the difference is not perceptible on a
throttled connection, this is complexity with nothing to show, and the right
outcome is to close this as dropped and say why.

## Acceptance criteria

- [ ] Hover prefetch with a short delay
- [ ] Viewport prefetch gated on connection quality
- [ ] `saveData` respected
- [ ] Bounded in-memory cache, no persistence
- [ ] A before-and-after measurement recorded on the item
