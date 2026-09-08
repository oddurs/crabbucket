---
id: 5
title: Heading ids and anchor links
type: feature
status: backlog
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: content
---

## Problem

No heading in these docs can be linked to. That is the difference between
documentation you can point a colleague at and documentation you can only
point at the top of.

It also blocks two other things: a table of contents has nothing to link to,
and fragment links cannot be checked because there is nothing to check them
against.

## Proposal

Slugify heading text into an id: lowercase, non-alphanumerics to hyphens,
collapse runs, trim. Disambiguate collisions within a page with `-2`, `-3`.

pulldown-cmark's `ENABLE_HEADING_ATTRIBUTES` is already on, so an explicit
`{#custom-id}` should win over the derived slug.

Render a permalink anchor inside the heading — visually hidden until the
heading is hovered or focused, `aria-label="Link to this section"`. Keyboard
reachable, not mouse-only.

Return the heading list from the markdown pass so the table of contents and
the fragment checker can both consume it without parsing HTML back.

## Acceptance criteria

- [ ] Every `h2`–`h4` gets a stable id
- [ ] Explicit `{#id}` overrides the derived one
- [ ] Duplicate headings on one page get distinct ids
- [ ] Ids are stable across builds — no content hashing
- [ ] `markdown::to_html` returns the headings alongside the HTML
