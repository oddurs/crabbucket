---
id: 19
title: Previous and next links within a section
type: feature
status: done
milestone: v0.2
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: ui
---

## Problem

Docs are read in order the first time and out of order afterwards. There is
currently no way to read them in order without going back to the index
between every page.

## Proposal

The `Docs` layout already has the section list. Render the neighbours of the
current page at the foot of the content, labelled with their titles rather
than with "Previous" and "Next" alone — the title is the useful half.

Ordering follows the sidebar, so this is a question of what orders the
sidebar, which is `nav_order` where present and route order otherwise. That
ordering rule should be stated once and shared, not reimplemented here.

Omit the link at each end rather than rendering a disabled one.

## Acceptance criteria

- [x] Neighbours rendered at the foot of `Docs` pages
- [x] Labelled with page titles
- [x] Ends omit the missing side
- [x] Ordering shared with the sidebar, not duplicated

## 2026-09-08

Done, and it turned up a design gap worth more than the feature.

The item said ordering follows "nav_order where present and route order otherwise". That rule was wrong: `nav_order` decides whether a page is in the masthead at all, so using it for reading order would have put every documentation page in the masthead. Sections now have their own `order` field, and `SiteIndex::under` sorts by it and falls back to route order. Previous and next are derived from exactly that list rather than reimplementing the rule, because two orderings that can disagree eventually will.

Before this the docs read alphabetically: Components, Content, Deploying, Design, Design systems... Now they read in the order somebody would actually want.
