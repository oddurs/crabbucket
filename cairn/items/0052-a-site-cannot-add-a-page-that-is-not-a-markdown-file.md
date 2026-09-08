---
id: 52
title: A site cannot add a page that is not a Markdown file
type: feature
status: done
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

- [x] A site crate can add a page whose body it rendered
- [x] Added pages are link-checked, indexed for search, and in the sitemap
- [x] They can appear in navigation with an order
- [x] The generated `Route` enum includes them
- [x] `examples/site-crate` has one, and it is the landing page

## 2026-09-08

Done. The hard part the item flagged -- that the generated Route enum has
to know about pages `build.rs` cannot see -- turned out to have a clean
answer: the site declares them in `site.toml`, which both halves read.

`[[page]]` gives a page its title, layout, navigation position and
everything else a written page gets from frontmatter. `Options::pages`
supplies the body. `build.rs` calls `crabbucket_routes::declared()` on the
same file, so the enum has them too.

That split is what makes the rest fall out. The declaration is the page's
identity and the body is only its content, so:

- Declaring without rendering is an error.
- Rendering without declaring is an error -- a body with no title, no
  layout and no place in the navigation is not a page.
- Declaring a route the content already has is an error rather than one of
  them silently winning.

The metadata is deserialized into the design system's own types, so a
declared page gets exactly the checking a written one does: an unknown
layout fails, a missing required field fails, and both name site.toml.

Two costs, both recorded in the docs. `crabbucket-routes` gained a
dependency on `toml`, so the "no dependencies" claim is now "one, and a
Markdown parser is still not welcome". And a declared page has no headings,
because nothing parsed its body -- so no table of contents, and a fragment
link into one cannot be checked.

examples/site-crate's landing page is now rendered from Rust, which is the
demonstration: it has no Markdown file and is still a route the compiler
knows about.
