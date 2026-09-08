+++
title = "Getting started"
layout = "docs"
+++

# Getting started

## Install

crabbucket needs Rust 1.85 or newer.

```sh
cargo install --git https://github.com/oddurs/crabbucket crabbucket-cli
```

That installs one executable, `crab`.

## A site is a directory

```
my-site/
  site.toml
  content/
    index.md
  static/            # optional; copied verbatim
```

`site.toml` is the whole configuration:

```toml
title = "My project"
description = "One line, used for meta description."

# Absolute URL, needed only for feeds and og: metadata. Optional.
url = "https://me.github.io/my-project/"

# The path the site is served from. A GitHub Pages project site lives
# under /repo/; a user site or a custom domain lives at /.
base = "/my-project/"

# Ship the ~1KB client router. Off by default.
router = true
```

And `content/index.md` is a page:

```markdown
+++
title = "My project"
layout = "landing"
nav_order = 1
+++

# My project

It does a thing.
```

## Build

```sh
crab build
```

```
crab: 4 pages, 11 links checked -> dist
```

`dist/` is a complete static site. Every route is a directory holding an
`index.html`, so every URL ends in a slash and works on any static host with
no rewrite rules.

## What `build` refuses to do

The build is a gate, not a formatter. It fails when:

- a page has no `title`, or its frontmatter does not fit the schema
- a page names a `layout` the theme does not have
- any page links to a route or an asset that does not exist

The last one is the interesting one. See [Routing](../routing/).

## Develop

There is no `crab dev`. Run [turborust](../dev-loop/) instead.
