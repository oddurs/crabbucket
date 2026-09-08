---
id: 19
title: Previous and next links within a section
type: feature
status: backlog
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

- [ ] Neighbours rendered at the foot of `Docs` pages
- [ ] Labelled with page titles
- [ ] Ends omit the missing side
- [ ] Ordering shared with the sidebar, not duplicated
