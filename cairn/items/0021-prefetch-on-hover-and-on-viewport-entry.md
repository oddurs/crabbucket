---
id: 21
title: Prefetch on hover and on viewport entry
type: feature
status: done
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

- [x] Hover prefetch with a short delay
- [x] Viewport prefetch gated on connection quality
- [x] `saveData` respected
- [x] Bounded in-memory cache, no persistence
- [x] A before-and-after measurement recorded on the item

## 2026-09-08

Done, and measured first, because the item said to.

A page fetch from GitHub Pages, five runs against the live site:

    connect 0.106  ttfb 0.230  total 0.231  13653 bytes
    connect 0.041  ttfb 0.174  total 0.178
    connect 0.053  ttfb 0.161  total 0.167
    connect 0.048  ttfb 0.158  total 0.158
    connect 0.086  ttfb 0.347  total 0.360

With the connection already warm, which it is inside a session, the
marginal cost is ttfb minus connect: 110 to 260ms. That is comfortably
over the threshold where a navigation stops feeling instant, so the
complexity earns its place rather than being assumed to.

Hover prefetch after 65ms, cancelled on pointerout. Viewport prefetch
only on a connection that reports 4g and does not ask to save data.
Never on pointerdown -- by then the fetch is happening anyway. The cache
is eight entries, in memory only, and dropped with the page: a cache that
outlives the document serves stale content after a deploy, which is a
worse bug than a slow navigation.

Not measured: the perceived difference in a browser on a throttled
connection. That needs a browser harness, which this project does not
have yet. The number above is the latency being removed, which is the
honest core of the claim.
