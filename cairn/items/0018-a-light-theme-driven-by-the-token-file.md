---
id: 18
title: A light theme, driven by the token file
type: feature
status: backlog
milestone: v0.2
labels:
- design
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: ui
---

## Problem

The design system is dark only. Half of readers set their system to light,
and a docs site that ignores that is a docs site people read in a squint.

More to the point: this is the test of whether the token mechanism actually
works. If a second palette cannot be expressed in the token file, the token
file is not the source of truth it claims to be.

## Proposal

Extend `design/tokens.toml` with a per-scheme layer:

    [color]
    surface = "#0b0d10"

    [color.light]
    surface = "#ffffff"

The build script emits the base palette on `:root`, and the light values
under both `@media (prefers-color-scheme: light)` and
`:root[data-theme="light"]`, so an explicit choice can win over the system
one. The Rust constants stay unchanged — they are already `var()`
references, which is precisely why this works without touching a single
component.

Then check the light palette properly rather than by eye: contrast ratios for
body text, muted text, links and accents against their own surfaces, asserted
in a test rather than sampled by hand.

## Acceptance criteria

- [ ] A light palette lives in the token file, not in hand-written CSS
- [ ] Both `prefers-color-scheme` and `data-theme` are honoured
- [ ] No component changes
- [ ] Body and muted text meet WCAG AA in both schemes, asserted by a test
- [ ] `color-scheme` is declared so form controls and scrollbars follow
