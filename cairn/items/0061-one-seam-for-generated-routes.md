---
id: 61
title: One seam for generated routes
type: feature
status: backlog
milestone: v1.1
depends_on:
- 59
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: content
---

## Problem

Three features are missing and they are the same feature: **taxonomies**
(a page per tag), **pagination** (a page per slice of a collection), and
**per-term feeds**. Every generator grows all three separately and ends up
with three configuration blocks, three URL schemes and three sets of edge
cases.

Zola's taxonomies are instructive. `TaxonomyTerm { name, slug, path,
permalink, pages }` — a term is a *route*, with a slug, a path and a
permalink, that no file on disk corresponds to. Its one really good idea is a
build failure: a term whose name slugifies to an empty string fails, naming
the term, rather than writing a page at `/tags//`.

crabbucket already has the machinery. `[[page]]` and `Options::pages()` exist
so a site can declare a page it renders itself, and those pages are checked
and linked exactly like any other. What is missing is not a taxonomy feature;
it is a typed API for *many* generated routes derived from a collection.

## Proposal

One seam, three instances.

A site crate declares a generator: something that takes the loaded collection
and returns routes, each with a typed value attached. Taxonomies, pagination
and per-term feeds are then written in a site's own code — or supplied as
small helpers in `crabbucket` — rather than configured.

The shape to aim for, roughly:

```rust sketch
pub trait Generate<T> {
    type Value;
    fn routes(&self, pages: &[Entry<T>]) -> Result<Vec<Generated<Self::Value>>>;
}
```

so that a generated route carries a value the design system receives typed,
the same way `PageMeta::extra` does. A tag page gets its term and its pages; a
paginated index gets its slice, its page number and its neighbours.

Constraints that come from the existing design:

- Every generated route goes through `Url`, so the base path stays solved.
- Every generated route joins the route set *before* link checking, so a link
  to `/tags/rust/` is checked like any other and a collision with a content
  file fails the build naming both.
- Slugification is a place a build can fail: an empty slug, or two terms
  slugifying to the same thing, are errors naming the terms — not silently
  merged, and not a page at a path with a doubled slash. `Url` cannot express
  the second one any more, but the collision still needs a name.
- Nothing generated may be in the primary navigation by accident.

This item is the seam plus taxonomies. Pagination and per-term feeds follow
as separate items once the seam has one real user, on the same principle that
gave the project a second design system.

## Acceptance criteria

- [ ] A typed generator API; generated routes carry a value the theme reads
- [ ] Taxonomies implemented on it, in `examples/site-crate`
- [ ] Generated routes are link-checked, and collide loudly with content
- [ ] An empty or duplicated slug fails the build naming the term
- [ ] The book gains a chapter, or an existing chapter gains the section
