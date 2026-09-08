---
id: 69
title: 'Endpoints: a typed value becomes a file'
type: feature
status: backlog
milestone: v1.2
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: build
---

## Problem

The build writes `search.json`, `sitemap.xml`, `feed.xml`, `atom.xml` and
`robots.txt`. Every one is a file generated from a typed value by
special-purpose code inside `site.rs`, and every one is hand-serialised.

A site that wants its own — a `versions.json` for a version switcher, an
`llms.txt`, a `.well-known` file, a JSON feed, a CSV of release dates — has
nowhere to put it except `static/`, written by hand, drifting from the content
it was derived from.

## Proposal

Make the thing the framework already does five times available once.

An endpoint is a route that produces bytes and a content type rather than a
page. A site declares one, gets the collection, and returns a value that is
serialised — so `versions.json` is `Serialize` on a Rust struct built from the
same collection the pages come from, and cannot disagree with them.

```rust sketch
pub trait Endpoint {
    fn route(&self) -> &str;
    fn render(&self, pages: &SiteIndex) -> Result<(Mime, Vec<u8>)>;
}
```

The gate that makes this worth having rather than a convenience: **an
endpoint's route joins the asset set, so a link to it is checked.** A page
linking `/versions.json` currently fails link checking unless the file is in
`static/`; an endpoint makes it a real, checked route. And a collision between
an endpoint and a page fails the build naming both.

Then the five built-ins become endpoints implemented on the same trait — which
is the real test of whether the trait is the right shape, and the same
argument that produced a second design system. If the sitemap cannot be
written as an endpoint, the trait is wrong.

This is also where the "web framework" line gets drawn in code rather than in
prose. An endpoint runs at build time and produces a file. There is no request,
no handler, no server. `/api/thing.json` is a static file that a CDN serves,
and the only thing that makes it an "API" is that something fetches it.

## Acceptance criteria

- [ ] An `Endpoint` trait; a site can add one
- [ ] Endpoint routes are link-checked, and collide loudly with pages
- [ ] The sitemap, feeds and search index reimplemented on it, or a written
      reason why one of them cannot be
- [ ] `doc/DESIGN` states that an endpoint is a file, not a handler
