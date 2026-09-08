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

## Start with a scaffold

```sh
crab new my-project --router
cd my-project
crab build
```

That writes a site that builds with no edits: a configuration file, three
pages, a `turborust.toml` for the dev loop, and a GitHub Pages workflow. It
refuses to write into a directory that already has anything in it, unless you
pass `--force`.

The rest of this page is what it wrote, and why.

## A site is a directory

```
my-site/
  site.toml
  content/
    index.md
    404.md           # optional; becomes 404.html
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

# Ship the client router: ~1.6KB gzipped. Off by default.
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

Two overrides exist for one invocation, so a preview never means editing a
tracked file:

```sh
crab build --out /tmp/preview --base /preview/
```

`dist/` is a complete static site. Every route is a directory holding an
`index.html`, so every URL ends in a slash and works on any static host with
no rewrite rules.

## What `build` refuses to do

The build is a gate, not a formatter. It fails when:

- a page has no `title`, or its frontmatter does not fit the schema
- a page names a `layout` the theme does not have
- any page links to a route, an asset, or a heading that does not exist
- the error page contains a relative link

Failures point at the line:

```
crab: content/docs/routing.md:3:10: unknown variant `dcos`, expected one of `page`, `docs`, `landing`
  |
3 | layout = "dcos"
  |          ^^^^^^
```

See [Routing](../routing/) for what link checking covers.

## Develop

There is no `crab dev`. Run [turborust](../dev-loop/) instead.
