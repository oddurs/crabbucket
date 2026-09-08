---
id: 10
title: sitemap.xml and robots.txt
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: build
---

## Problem

Nothing tells a crawler what the site contains. For a docs site that is the
difference between being findable and not.

## Proposal

Both are trivial given `config.url` and the route set, and both should be
skipped rather than guessed when `url` is absent — a sitemap full of wrong
absolute URLs is worse than no sitemap.

- `sitemap.xml` — every non-draft, non-404 route as an absolute URL. No
  `lastmod` until there is a real source for it; a fabricated timestamp is
  worse than an absent field.
- `robots.txt` — allow everything, point at the sitemap.

Add both to the asset set so link checking knows they exist.

## Acceptance criteria

- [x] `sitemap.xml` lists every live route, absolute
- [x] Neither file is emitted when `url` is unset
- [x] `robots.txt` references the sitemap
- [x] Drafts and the 404 page are excluded
- [x] Both are registered as assets

## 2026-09-08

Done. Both skipped entirely when site.toml has no url, rather than emitted with guessed absolute URLs. No lastmod: there is no honest source for one. Caught a double-base-path bug in review -- url already contains the base, so appending it again produced /newsite/newsite/. There is now a test named after that bug.
