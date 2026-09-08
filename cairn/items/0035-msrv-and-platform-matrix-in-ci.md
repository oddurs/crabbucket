---
id: 35
title: MSRV and platform matrix in CI
type: chore
status: backlog
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: ci
---

## Problem

`rust-version = "1.85"` is declared and never tested. CI runs stable on
Ubuntu only, so the claim is unverified and the two other platforms people
actually use are unexercised — including Windows, where every path assumption
in the build goes to die.

## Proposal

A matrix: stable and the declared MSRV, across Linux, macOS and Windows.

Windows is the one that will find bugs. `route_of` joins components with `/`,
`tree()` replaces backslashes, and link resolution assumes forward slashes
throughout. At least one of those is wrong, and a fixture site with nested
content will say which.

Keep the matrix small enough to stay fast: MSRV on Linux only, stable
everywhere. Add a scheduled run against beta, so a regression in a future
compiler arrives as a notification rather than as a surprise during a
release.

## Acceptance criteria

- [ ] Stable on Linux, macOS and Windows
- [ ] MSRV verified on Linux
- [ ] Windows path handling covered by a nested-content fixture
- [ ] Scheduled beta run
- [ ] Total CI time under five minutes
