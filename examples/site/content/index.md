+++
title = "A site is a typed value"
nav_order = 1
nav_label = "Home"
+++

# crabbucket

A static site framework in Rust for landing pages and documentation — the
kind that live next to a repository and get deployed to GitHub Pages.

Most site generators are string pipelines. Templates render strings,
frontmatter is a loose map, links are strings, design tokens are strings,
the base path is a string. Every one of those is a place a small site
breaks in a way you only find in production.

**The build either fails, or the site is correct.**

## What that buys you

| Usually a runtime surprise | Here |
|---|---|
| A link to a page that was renamed | A compile error |
| A page with no `title` | A build error naming the file and line |
| A design token renamed six months ago | A compile error at every use site |
| A stylesheet 404ing under `/repo/` | Impossible; the base path is applied in one place |

## Getting started

```sh
cargo install --git https://github.com/oddurs/crabbucket crabbucket-cli
crab build
```
