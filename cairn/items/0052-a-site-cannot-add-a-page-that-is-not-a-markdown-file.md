---
id: 52
title: A site cannot add a page that is not a Markdown file
type: feature
status: backlog
milestone: v1.0
labels:
- migration
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: routing
---

## Problem

Every route comes from a file in `content/`. There is no way to add a page
whose body is written in Rust.

Migrating cairn's site, four of its ten pages were `.astro` files rather than
content: the landing page, and three that compose prose with layout in ways a
Markdown file cannot express. They have nowhere to go. The workaround is to
rewrite each as Markdown plus directives, which loses the thing that made them
`.astro` files in the first place.

This is the largest gap the migration found, and it is a gap against the
framework crabbucket takes its model from — Astro has both content collections
and `.astro` pages, and crabbucket has only the first.

## Proposal

A site crate can already call `crabbucket::build`. It should be able to hand it
pages it rendered itself:

    crabbucket::build_with(dir, &theme, &Options {
        pages: vec![Page::at("", landing(&config))],
        ..Options::default()
    })?;

Those pages join the route set, are link-checked like any other, and appear in
the navigation and the sitemap. The `Route` enum should know about them too,
which means the generator needs to be told — probably by the site declaring
them somewhere both `build.rs` and `main.rs` can read.

That last part is the hard bit and the reason this is not small.

## Acceptance criteria

- [ ] A site crate can add a page whose body it rendered
- [ ] Added pages are link-checked, indexed for search, and in the sitemap
- [ ] They can appear in navigation with an order
- [ ] The generated `Route` enum includes them
- [ ] `examples/site-crate` has one, and it is the landing page
