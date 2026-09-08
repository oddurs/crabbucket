---
id: 49
title: Report is the model for what a library API should hand back
type: chore
status: backlog
labels:
- api
created: 2026-09-08
updated: 2026-09-08
priority: p3
effort: s
area: build
---

## Problem

An external review found two API papercuts on the same day:

- `build_with` and `Options` were reachable at `crabbucket::site::*` but not
  re-exported at the root beside `build`, so the symmetric name did not
  compile and the reviewer wrote a shim that moved `dist/` by hand.
- `Report` had no `Display`, so the first thing anybody writes printed neither
  the warnings nor the draft count. A warning printed to nobody is a warning
  that was not printed.

Both are fixed. Both were the same mistake: the correct thing was not the easy
thing, and nothing in the crate made the omission visible.

## Proposal

Not a feature. A standing check to run before 1.0, when the API stops being
free to change:

- Every type a caller has to construct or read is re-exported at the root.
- Every type a caller receives can be printed usefully without their writing
  the formatting.
- Anything easy to have and easy to forget to print is in that output.

Fold it into the stability document (0033) rather than leaving it here.

## Acceptance criteria

- [ ] The three rules above are in `doc/STABILITY`
- [ ] The root re-exports are audited once against them
- [ ] Anything that fails the audit is fixed or written down as deliberate
