---
id: 16
title: Static search index and a small client
type: feature
status: backlog
milestone: v0.2
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: ui
---

## Problem

A docs site without search is a docs site you read once. This is the single
feature most responsible for whether documentation gets used twice.

## Proposal

Build time: emit `search.json` — per page, the route, the title, the section
headings, and the body text stripped of markup. For a site of tens of pages
this is tens of kilobytes; do not build an inverted index for a corpus that
small, because a linear scan over it is genuinely faster than parsing the
index would be.

Client: a small script, in the same spirit as the router — fetch the index on
first interaction rather than on page load, match on title and headings first
and body text second, render results under the input.

Keyboard first: `/` to focus, arrows to move, enter to go, escape to close.
The input is a real `<form>` with a real `<input>`, so with no JavaScript it
degrades to nothing rather than to something broken.

Cap the index: warn during the build if it exceeds a few hundred kilobytes,
because that is the point at which this design stops being the right one and
the honest thing is to say so rather than to ship a slow page.

## Acceptance criteria

- [ ] `search.json` emitted with route, title, headings, and text
- [ ] Index fetched lazily, not on page load
- [ ] Title and heading matches rank above body matches
- [ ] Full keyboard operation
- [ ] Absent JavaScript, the search UI is simply not shown
- [ ] Build warns when the index grows past the size this design suits
