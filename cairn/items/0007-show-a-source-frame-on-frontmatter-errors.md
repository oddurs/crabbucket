---
id: 7
title: Show a source frame on frontmatter errors
type: feature
status: backlog
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: build
---

## Problem

`toml` reports a line and a column, and the error currently prints them as
prose. rustc taught everyone what a good error looks like, and it is not
prose — it is the offending line with a caret under it.

Compare:

    crab: content/docs/routing.md: TOML parse error at line 2, column 1

with:

    crab: content/docs/routing.md:2:1: missing field `title`
      |
    2 | ttile = "Routing"
      | ^^^^^ unknown field

## Proposal

`toml::de::Error` exposes a span. The file text is already in hand at the
point the error is constructed, so build the frame there rather than trying
to reconstruct it later.

Offset the reported line by the frontmatter's own offset in the file, so the
number matches what an editor shows — this is the bug every frontmatter
parser ships with, and it is worth a test of its own.

Keep the plain form when stderr is not a terminal, and keep the first line
in the `file:line:col: message` shape editors already know how to jump to.

## Acceptance criteria

- [ ] Errors show the offending line with a caret span
- [ ] Line numbers are file-absolute, not frontmatter-relative
- [ ] `file:line:col:` prefix on the first line
- [ ] Colour only when stderr is a terminal
- [ ] A test asserts the line number against a page with a long body above
