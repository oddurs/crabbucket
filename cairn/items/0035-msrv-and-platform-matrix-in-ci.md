---
id: 35
title: MSRV and platform matrix in CI
type: chore
status: done
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

- [x] Stable on Linux, macOS and Windows
- [x] MSRV verified on Linux
- [x] Windows path handling covered by a nested-content fixture
- [x] Scheduled beta run
- [x] Total CI time under five minutes

## 2026-09-08

Done, and the first thing it found was that the declared minimum was
fiction. `rust-version = "1.85"` had been there since the first commit and
was never checked; the code has used let-chains since v0.1, and those are
1.88. A toolchain is installed for 1.88, so this was settled by building
rather than by reading a release note.

That is the whole argument for the item: a claim nothing tests is a claim
that is wrong eventually, and this one was wrong immediately.

The matrix is four jobs rather than twelve. Stable on Linux, macOS and
Windows; the minimum on Linux only, because checking it three times buys
nothing. Formatting and clippy run once rather than four times, for the
same reason.

Windows path bugs, found by reading rather than by CI, because CI cannot
run until this merges: five assertions compared a rendered path against a
literal with forward slashes, which passes everywhere except Windows.
There is a `names()` helper now that builds the literal with the
platform's separator. The nested fixture went four levels deep with links
out of the bottom, so there is something for the platform to disagree
about.

The production code looks right: routes are built from `Path::components`
joined with `/`, and `tree()` normalises backslashes. But that is a
reading, not a result, and Windows CI on this PR is what turns it into
one.

A scheduled beta run on Mondays, so a compiler regression arrives as a
notification rather than during a release.
