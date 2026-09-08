---
id: 26
title: Document building a design system from scratch
type: docs
status: backlog
milestone: v0.3
depends_on:
- 25
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: docs
---

## Problem

The design-systems page explains the mechanism — tokens, components, a theme
crate — but does not walk anyone through building one. The claim that
restyling a fleet is a version bump is currently a claim, not a tutorial.

## Proposal

A page that builds a small theme end to end:

1. A new crate, a token file, the build script that generates from it.
2. The `Layout` enum, and why it is an enum.
3. A page shell, one component, and the styles beside it.
4. Depending on it from a site.
5. Publishing it privately, and what a version bump does to twelve sites.

Written against `crabbucket-theme-plain`, so every snippet is code that
exists and is compiled by CI rather than prose that resembles code.

Include the parts that are annoying: what happens when the theme adds a
layout a site does not use, what happens when a token is removed, and how to
migrate a site across a breaking theme change.

## Acceptance criteria

- [ ] A complete walkthrough, end to end
- [ ] Every snippet drawn from a compiled crate
- [ ] Covers adding and removing tokens and layouts
- [ ] Covers migrating a site across a breaking change
- [ ] Linked from the design-systems page
