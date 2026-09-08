+++
title = "Reference"
description = "Frontmatter, site.toml, the CLI, the Theme trait and the errors, on one page."
layout = "docs"
order = 9
+++

# 9. Reference

Everything on one page, for the tab you keep open. The
[documentation](~/docs/) has the long form of each of these.

## Frontmatter

Between `+++` fences at the top of every content file.

| Field | Type | Meaning |
|---|---|---|
| `title` | string | Required. Used in `<title>` and as the default nav label. |
| `description` | string | Falls back to the site description. |
| `layout` | the design system's layout type | Which layout renders this page. Unknown names fail the build. |
| `nav_order` | number | Put this page in the site navigation, at this position. |
| `nav_label` | string | A shorter label for navigation than the title. |
| `order` | number | Sort position within this page's own section. |
| `date` | date | A real TOML date. Required for pages in a configured feed. |
| `draft` | bool | Skip the page entirely. |

Plus whatever the design system's `Extra` type declares.

## `site.toml`

| Key | Type | Meaning |
|---|---|---|
| `title` | string | Required. |
| `description` | string | Used where a page has none. |
| `base` | string | Where the site is served from. Defaults to `/`. |
| `url` | string | Absolute site address. Turns on the sitemap, `robots.txt`, feeds and social cards. |
| `search` | bool | Ship the search index and client, if the design system has one. |
| `router` | bool | Ship the client-side router, if the design system has one. |

```toml
[[feed]]
collection = "notes"
title = "Notes"
limit = 20
```

```toml
[[page]]
route = "changelog"
title = "Changelog"
```

`[[page]]` declares a page the site renders itself, in Rust, rather than from
a Markdown file. It is checked and linked exactly like any other page.

## The command line

```
crab build [DIRECTORY]     build the site in DIRECTORY (default: .)
crab new DIRECTORY         scaffold a new site

Options for build:
      --out PATH           write the site to PATH instead of dist/
      --base PATH          serve from PATH instead of the configured base

Options for new:
      --base PATH          the path the site will be served from
      --theme NAME         depend on a design system, as a crate name or a
                           git URL; the site is then a crate
      --router             ship the client-side router
      --force              scaffold into a directory that is not empty
```

Exit status: **0** on success, **1** if the site is wrong, **2** if the
command line is.

There is no dev server. Run `turborust up`.

## The `Theme` trait

```rust sketch
pub trait Theme {
    type Layout: DeserializeOwned + Default;
    type Extra: DeserializeOwned + Default;

    fn render(&self, page: &Page<'_, Self>) -> String where Self: Sized;
    fn stylesheet(&self) -> String;

    fn directives(&self) -> Directives { Directives::new() }
    fn og_image(&self, page: &Page<'_, Self>) -> Option<Vec<u8>> where Self: Sized { None }
    fn search_js(&self) -> Option<String> { None }
    fn router_js(&self) -> Option<String> { None }
}
```

Two required methods. The four defaults each say "not this design system",
and a site that asks for one anyway is warned and still builds.

## What a `Page` carries

`config`, `meta`, `route`, `html`, `headings`, `site`, `feeds`, `cache`,
`card`.

And three methods worth knowing:

| Call | Gives you |
|---|---|
| `page.nav()` | Pages with a `nav_order`, in that order, current marked |
| `page.section("docs")` | Pages under `docs/`, in `order`, current marked |
| `page.neighbours("docs")` | The two either side, in the same order |

## Building from Rust

```rust sketch
crabbucket::build(site_dir, &Theme)?;

crabbucket::build_with(
    site_dir,
    &Theme,
    Options::new().out_dir(out).maybe_base(base),
)?;
```

`Report` carries `routes`, `out_dir`, `drafts`, `links`, `warnings` and
`timings`. Printing it says all of it:

```
17 pages, 730 links checked -> dist
warning: search.js was written but no page loads it
```

## Links

| Written | Checked as |
|---|---|
| `../install/` | Relative to this page's URL; a route |
| `/ferrite/docs/` | Site-absolute; must start with the base |
| `~/docs/` | The site root, whatever the base is |
| `logo.svg` | An asset: `static/`, or a file the build emits |
| `#install` | A heading id on this page |
| `../install/#cargo` | A heading id on that page |
| `https://…`, `mailto:`, `#` | Not checked |

## Fence flags

| Fence | Effect |
|---|---|
| ```` ```rust ```` | Highlighted at build time |
| ```` ```console no-search ```` | Kept out of the search index |

## Errors that stop the build

- frontmatter that does not fit the type, with the file and the line
- an unknown layout, listing the ones that exist
- an unknown directive, or an attribute of the wrong type
- two files claiming the same route, naming both
- a dead internal link or fragment, naming every one
- an undated page in a configured feed
- a palette that fails WCAG AA contrast, in either scheme

## Warnings that do not

- a feed configured with no `url`
- `search` or `router` asked for by a design system that has neither
- a client written that no page loads
- a search index that has outgrown being one file

## Where the time goes

On 521 pages: read and parse 195ms, write 43ms, render 7ms, check links 2ms.
Highlighting is about sixty percent of the whole build. The numbers, and how
they compare to Hugo and Zola, are on the [Speed](~/docs/speed/) page.
