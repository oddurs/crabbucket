---
id: 18
title: A light theme, driven by the token file
type: feature
status: done
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

- [x] A light palette lives in the token file, not in hand-written CSS
- [x] Both `prefers-color-scheme` and `data-theme` are honoured
- [x] No component changes
- [x] Body and muted text meet WCAG AA in both schemes, asserted by a test
- [x] `color-scheme` is declared so form controls and scrollbars follow

## 2026-09-08

Done, and it worked exactly as the design promised: a second palette cost zero component changes, because the Rust constants are var() references and only the custom properties move.

A `light` sub-table under any token group overrides that group. build.rs asserts every light key has a base key to override, so a token that exists in only one scheme fails the build rather than falling back to nothing. color-scheme moved out of hand-written CSS into the generated block, so it flips with the palette.

The media query is guarded as :root:not([data-theme="dark"]) and the attribute selector comes last, so an explicit choice beats the system preference in both directions.

Contrast is a test rather than a judgement: crabbucket-ui/tests/contrast.rs computes WCAG ratios over the generated token tables and holds text, muted text, links and accents to AA on both surfaces in both schemes, syntax colours to AA-large, and borders to a band. The hand-picked palette passed on the first run. The only failure was my own sanity check, which claimed #777777 clears AA on white; it does not, at 4.48:1, and the test now asserts that boundary in both directions.
