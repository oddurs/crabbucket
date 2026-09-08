---
id: 20
title: Copy button on code blocks
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

Every code block in these docs is meant to be run, and every one of them has
to be selected by hand.

## Proposal

Only when the site opts into script — the same switch as the router. A
button positioned inside the block's corner, appearing on hover and always
present to keyboard focus, `aria-label="Copy code"`, with a transient
confirmation that is announced rather than only shown.

Strip a leading shell prompt (`$ `) when copying, because copying the prompt
is the small annoyance this feature exists to remove.

## Acceptance criteria

- [x] Button on every fenced block, when script is enabled
- [x] Keyboard reachable, not hover-only
- [x] Confirmation announced to assistive technology
- [x] Leading `$ ` stripped
- [x] Nothing added when the router is off

## 2026-09-08

Done. The button is created by the router rather than emitted by the build, so a reader with scripting off gets no button rather than a button that does nothing.

Keyboard reachable, `aria-label="Copy code"`, and the confirmation goes through the router's existing live region as well as changing the button text, so it is announced and not only shown. A leading `$ ` is stripped. Clipboard failure -- an insecure context, a denied permission -- falls back to telling the reader to press Ctrl-C rather than silently doing nothing.

The button's styles became a component of their own with the class templated into the router, like the nav and contents selectors. A hard-coded `cb-copy` in a file the theme supplies would have broken the moment a second theme existed.
