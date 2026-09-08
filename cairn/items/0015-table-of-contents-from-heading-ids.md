---
id: 15
title: Table of contents from heading ids
type: feature
status: done
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

- [x] `h2` and `h3` appear, nested correctly
- [x] Works with JavaScript disabled
- [x] Sticky on wide viewports, collapsed on narrow
- [x] Hidden on pages with fewer than three headings
- [x] Scroll-spy is opt-in with the router and degrades silently

## 2026-09-08

Done. Page now carries its headings, which are the same list the link checker validates fragments against -- so a contents entry cannot point at a heading that is not there, by construction rather than by care.

Rendered twice, as a sidebar and as a <details>, with CSS choosing. That costs a few hundred bytes and buys a layout that needs neither script nor knowledge of the viewport at build time. Skipped below three h2/h3 headings: a contents list with two entries takes a column to tell the reader what the page already told them.

The nesting loop is plain Rust rather than a template construct. The first version used maud's @while with a @let hack to advance the index; it compiled and read terribly. Building the items first and interpolating them is three lines longer and obviously correct.

Scroll-spy is in the router, so it ships only with the router, and the class it looks for is templated in like the nav selector rather than hard-coded -- a second theme would have broken the hard-coded one.
