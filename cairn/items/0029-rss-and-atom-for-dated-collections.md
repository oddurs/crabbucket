---
id: 29
title: RSS and Atom for dated collections
type: feature
status: done
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

- [x] Feeds generated from a configured collection
- [x] Dates come from that collection's own frontmatter type
- [x] Both RSS and Atom, both validating
- [x] `<link rel="alternate">` in the head
- [x] Skipped with a build message when `url` is unset

## 2026-09-08

Done, but not the way the item proposed, and the difference is worth
recording.

The item wanted a separate opt-in collection with its own frontmatter
type, as an exercise for the typed-collection API. That would have loaded
and rendered every dated page twice -- once as PageMeta to render it, once
as the feed's own type -- and rendering is the expensive part now that it
highlights. So `PageMeta` gained an optional `date` instead.

The exercise still happened, in a better place: `date` is a
`toml::value::Datetime`, so TOML's own date type does the parsing and a
malformed date fails at the file and the line with nothing in the feed
code doing any checking. That is the typed-collection idea applied to one
field rather than to a parallel type.

Configuring a feed is how a site says a collection is dated. An undated
page in one fails the build naming itself, which is the guarantee the
separate type would have given, without the second render.

Two decisions worth knowing:

Atom's channel-level `updated` is the newest entry's date, not the build
time. Using "now" would make every build differ from the last for no
reason and every reader think something had changed.

A date with no time is midnight UTC. Reading the build machine's zone
would make the same content produce different feeds on different machines.

The RSS date format needed a real weekday calculation -- Sakamoto's
method, with five known dates as anchors including a leap day and 1900,
which is not a leap year.

The link checker caught the feed files not being registered as assets,
before any of it was written down.
