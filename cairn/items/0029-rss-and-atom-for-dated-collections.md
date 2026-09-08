---
id: 29
title: RSS and Atom for dated collections
type: feature
status: backlog
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: build
---

## Problem

A project site eventually wants a changelog or a notes section, and a
section without a feed is a section nobody follows.

## Proposal

Feeds need dates, and the built-in `PageMeta` has none — deliberately, since
most pages are not dated. So this needs an opt-in collection with its own
frontmatter, which is a good exercise for the typed-collection API: if
generating a feed from a custom collection is awkward, the API is wrong.

Configure per collection:

    [[feed]]
    collection = "notes"
    title = "Notes"
    limit = 20

Emit both RSS 2.0 and Atom, since readers disagree about which they want, and
both are small. Link them from `<head>` on the collection's index page.

Absolute URLs mean this requires `url`; skip the feed rather than emit
relative links, and say so during the build.

## Acceptance criteria

- [ ] Feeds generated from a configured collection
- [ ] Dates come from that collection's own frontmatter type
- [ ] Both RSS and Atom, both validating
- [ ] `<link rel="alternate">` in the head
- [ ] Skipped with a build message when `url` is unset
