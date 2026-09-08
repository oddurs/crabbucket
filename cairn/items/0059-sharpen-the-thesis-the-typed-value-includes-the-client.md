---
id: 59
title: 'Sharpen the thesis: the typed value includes the client'
type: docs
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: docs
---

## Problem

`doc/DESIGN` says a site is a typed value, and it is right about everything
that happens before the browser opens the page. After that it stops being
true, and the document does not admit it.

The search box is hand-written JavaScript. It reads `search.json`, a file
nothing validates, whose shape is agreed between a Rust function and a
JavaScript function by hand. The router is 2.9KB of the same. If the index
format changes, the client breaks at runtime, in a browser, in front of a
reader — which is precisely the class of failure this project exists to
refuse, sitting inside the project.

That is not a bug to fix in a patch. It is the boundary of the concept, and
it is currently in the wrong place.

## Proposal

Extend the thesis, in writing, before building anything against it:

> Everything the browser receives was checked by the compiler — including the
> data that crosses to an interactive component, and the files a static
> endpoint serves.

And state where it stops, just as plainly: **there is no server.** No SSR, no
middleware, no runtime. Every route remains a file that any static host serves
without configuration. Dynamic data is fetched at build time by a typed
source, or in the browser by a typed island, and never by a crabbucket
process listening on a port.

That second half is the more important one. "Web framework" is a phrase that
usually smuggles a server in with it, and the deploy story — a directory, any
host, no runtime — is worth more than anything a server would add.

New sections in `doc/DESIGN`:

- the client boundary, and why it is the last untyped seam
- what a typed boundary buys that a validated one does not: Astro serialises
  island props through a hand-written twelve-entry tag table
  (`PROP_TYPE = { Value, JSON, RegExp, Date, Map, Set, BigInt, URL,
  Uint8Array, Uint16Array, Uint32Array, Infinity }`) with cyclic-reference
  detection that throws during render. A serde derive has no table to fall
  off and no cycle to detect.
- the comparison that positions the project: Hugo and Zola have no client
  story; Astro has one that is validated at runtime; Leptos and Dioxus have
  typed boundaries and no content pipeline. The combination is empty.
- what stays excluded, and why: a server, a plugin ABI, a template language.

## Acceptance criteria

- [ ] `doc/DESIGN` states the extended thesis and where it stops
- [ ] The no-server position is argued rather than assumed
- [ ] The competitive position is stated with specifics, not adjectives
- [ ] The Design page on the site follows
- [ ] `doc/STABILITY` says which of the new surfaces will be covered by 1.0
      promises and which will not
