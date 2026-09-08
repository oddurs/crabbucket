---
id: 20
title: Copy button on code blocks
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

- [ ] Button on every fenced block, when script is enabled
- [ ] Keyboard reachable, not hover-only
- [ ] Confirmation announced to assistive technology
- [ ] Leading `$ ` stripped
- [ ] Nothing added when the router is off
