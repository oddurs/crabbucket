---
id: 15
title: Table of contents from heading ids
type: feature
status: backlog
milestone: v0.2
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: ui
---

## Problem

The longer docs pages are already past the length where a reader can see the
shape of the page. There is no way to jump within one, and no way to see
what a page covers without scrolling it.

## Proposal

The markdown pass returns headings once heading ids land, so this is a theme
concern rather than a parsing one.

- `Page` exposes the heading tree.
- The `Docs` layout renders `h2`/`h3` as a second column, sticky, on wide
  viewports; collapsed into a `<details>` above the content on narrow ones.
- Highlight the section in view with an `IntersectionObserver`, but only when
  the router is enabled — the table of contents itself must work as plain
  anchor links with no script at all.

Skip the table of contents when a page has fewer than three headings; a
contents list with two entries is noise.

## Acceptance criteria

- [ ] `h2` and `h3` appear, nested correctly
- [ ] Works with JavaScript disabled
- [ ] Sticky on wide viewports, collapsed on narrow
- [ ] Hidden on pages with fewer than three headings
- [ ] Scroll-spy is opt-in with the router and degrades silently
