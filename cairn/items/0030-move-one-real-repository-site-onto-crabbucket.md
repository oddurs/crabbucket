---
id: 30
title: Move one real repository site onto crabbucket
type: chore
status: backlog
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: docs
---

## Problem

`examples/site` is the only site crabbucket has ever built, and it was
written by the same person, at the same time, against the same assumptions.
It cannot surface the things a real site would.

## Proposal

Pick an existing repository with a GitHub Pages site and move it over. The
turborust site is the obvious candidate, since the two projects already point
at each other.

Keep a log of every place the migration was harder than it should have been —
each of those is a real issue, filed against whichever milestone it belongs
to, and worth more than any amount of speculating about what a second user
would want.

The success condition is not "it looks the same". It is: the new site is
correct, and the number of things that had to be worked around is small
enough to write down.

## Acceptance criteria

- [ ] One real site migrated and deployed
- [ ] Every workaround filed as its own item
- [ ] Both sites share a theme crate
- [ ] The migration written up as a docs page
- [ ] Nothing in the migration required patching crabbucket locally
