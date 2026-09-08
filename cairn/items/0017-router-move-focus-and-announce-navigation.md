---
id: 17
title: 'Router: move focus and announce navigation'
type: bug
status: backlog
milestone: v0.2
labels:
- accessibility
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: router
---

## What happens

The client router swaps `<main>` and updates `document.title`, and that is
all. A screen reader user gets no announcement that anything happened, and
keyboard focus stays on the link that was activated — which no longer exists,
so focus falls back to `<body>` and the next Tab starts from the top of the
page.

This is the standard defect of every hand-rolled SPA router, and it is the
reason "no full page reload" is often a downgrade rather than an upgrade.

## What should happen

A client-side navigation should be at least as good as the full page load it
replaced.

- Move focus to the new `<main>` (`tabindex="-1"`, focus, do not scroll on
  focus).
- Announce the new page title through a polite live region.
- Preserve the fragment: navigating to `#section` should focus that heading,
  not the top.
- Restore scroll position on `popstate` rather than always scrolling to top —
  going *back* should land where the reader was.
- Respect `prefers-reduced-motion` for the view transition. The stylesheet
  already does; the script should not start one at all.

## Reproduction

1. Build `examples/site` with `router = true`
2. Tab to a nav link, press Enter
3. Press Tab — focus is at the top of the document, not in the new content
4. With VoiceOver on, nothing is announced

## Acceptance criteria

- [ ] Focus moves into the new main
- [ ] Title announced via a live region
- [ ] Fragment targets are focused
- [ ] Back restores scroll position
- [ ] No view transition started under `prefers-reduced-motion`
- [ ] Router stays under 2KB after all of it
