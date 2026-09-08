---
id: 26
title: Document building a design system from scratch
type: docs
status: done
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

- [x] A complete walkthrough, end to end
- [x] Every snippet drawn from a compiled crate
- [x] Covers adding and removing tokens and layouts
- [x] Covers migrating a site across a breaking change
- [x] Linked from the design-systems page

## 2026-09-08

Done, written against crabbucket-theme-plain as the item asked.

The criterion that mattered was "every snippet drawn from a compiled
crate", because that is the one that rots. Comparing snippets character by
character would break on any rustfmt reflow and on the guide's deliberate
`/* … */` elisions, so the test checks the load-bearing part instead:
every identifier the guide puts in front of a reader still exists in the
crate, spelled the way the guide spells it. Fifteen of them, plus the
dependency list, plus the link from the page it expands on, plus the five
awkward-part headings the item named.

Writing the test found that my first version counted `version.workspace =
true` in `[package]` as a dependency. It now parses the dependency tables.

The section the item was really asking for is "The parts that are
annoying": adding a layout is free, removing one is breaking, adding a
token is free, renaming one is breaking for the design system's own
components and free for every site -- because tokens are internal and
components and layouts are the public surface. That asymmetry is the thing
nobody says out loud, and it is what makes "restyling twelve sites is a
version bump" true rather than aspirational.

One place the design has no guarantee to offer, and the guide says so: a
site's own CSS written against a component's class names. Nothing checks
that, so class names are public.
