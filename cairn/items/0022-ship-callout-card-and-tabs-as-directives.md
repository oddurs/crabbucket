---
id: 22
title: Ship callout, card and tabs as directives
type: feature
status: backlog
milestone: v0.2
depends_on:
- 14
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: ui
---

## Problem

`crabbucket-ui` has exactly one component beyond the page shell, and no
Markdown page can reach it. Once directives land, the design system needs
enough components for the docs site to stop being a wall of prose.

## Proposal

The set a documentation site actually needs, and no more:

- `callout` — note, warn, and a `title` attribute
- `card` and `cards` — a grid of links, for index pages
- `tabs` — the same instructions for macOS, Linux and Windows, or for two
  package managers
- `steps` — an ordered walkthrough with visible numbering

`tabs` is the only one with real design questions: it needs script for the
switching, must show every panel with script off, and should remember the
chosen tab across pages within a session, since a reader on Linux does not
want to pick Linux on every page.

Each component gets a props type, so every attribute is checked, and each is
documented on the site by using it.

## Acceptance criteria

- [ ] Four directive families registered by the default theme
- [ ] Every attribute deserializes into a typed props struct
- [ ] `tabs` shows all panels with script disabled
- [ ] `tabs` remembers the selection within a session
- [ ] Each component documented on the site by being used there
