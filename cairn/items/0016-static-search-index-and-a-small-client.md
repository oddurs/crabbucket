---
id: 16
title: Static search index and a small client
type: feature
status: done
milestone: v0.2
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: ui
---

## Problem

A docs site without search is a docs site you read once. This is the single
feature most responsible for whether documentation gets used twice.

## Proposal

Build time: emit `search.json` — per page, the route, the title, the section
headings, and the body text stripped of markup. For a site of tens of pages
this is tens of kilobytes; do not build an inverted index for a corpus that
small, because a linear scan over it is genuinely faster than parsing the
index would be.

Client: a small script, in the same spirit as the router — fetch the index on
first interaction rather than on page load, match on title and headings first
and body text second, render results under the input.

Keyboard first: `/` to focus, arrows to move, enter to go, escape to close.
The input is a real `<form>` with a real `<input>`, so with no JavaScript it
degrades to nothing rather than to something broken.

Cap the index: warn during the build if it exceeds a few hundred kilobytes,
because that is the point at which this design stops being the right one and
the honest thing is to say so rather than to ship a slow page.

## Acceptance criteria

- [x] `search.json` emitted with route, title, headings, and text
- [x] Index fetched lazily, not on page load
- [x] Title and heading matches rank above body matches
- [x] Full keyboard operation
- [x] Absent JavaScript, the search UI is simply not shown
- [x] Build warns when the index grows past the size this design suits

## 2026-09-08

Done. The index is the pages, scanned linearly: for tens of pages that is
faster than parsing an inverted index would be, and a tenth of the code.
33KB for this site, 12KB over the wire. Past 300KB the build warns rather
than quietly shipping a slow page, and that threshold is a function that
can be tested rather than a comparison buried in the build.

Two extraction bugs, both found by reading the built index rather than by
a test, and both now tested:

Every tag was treated as a word boundary. Syntax highlighting wraps every
token in a span, so `serde::Deserialize` was being indexed as three words
and was unfindable. Inline tags no longer separate; block tags still do.

Heading permalinks were in the index. The `#` beside every heading landed
between the heading and the paragraph after it.

The JSON is written by hand -- three string fields and an array -- with
escaping that covers what the specification requires and leaves UTF-8
alone. The integration test parses the built index to confirm it is real
JSON rather than merely plausible.

The index URL comes from a data attribute on the form rather than being
substituted into the client, because it carries the site's base path and
the client should not know there is one. The first version substituted a
placeholder that was never replaced; the test that now checks both
clients for leftover placeholders would have caught it.
