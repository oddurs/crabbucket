+++
title = "Deploying"
layout = "docs"
order = 8
+++

# Deploying

`dist/` is a directory of static files with no server requirements. Any host
will serve it. GitHub Pages is the one crabbucket is shaped around.

## What the build writes

```
dist/
  index.html
  docs/index.html      every route is a directory
  404.html             a file, because that is what hosts look for
  site.css
  router.js            only if the site asked for it
  sitemap.xml          only if site.toml has a url
  robots.txt           only if site.toml has a url
  .nojekyll
```

The sitemap and `robots.txt` need absolute URLs, so they are skipped entirely
when `url` is unset rather than emitted with guessed ones. The sitemap carries
no `lastmod`: there is no honest source for one, and a fabricated timestamp is
worse than an absent field.

## The two things that make it work

**`base`.** A project site is served from `https://you.github.io/repo/`, so
`base = "/repo/"` in `site.toml`. A user site or a custom domain is served
from the root, so `base = "/"`. Set it once; nothing else in the site knows.

**`.nojekyll`.** The build writes an empty one into `dist/`. Without it,
GitHub Pages runs the output through Jekyll, which silently drops every file
and directory whose name begins with an underscore. This is a famously
annoying thirty minutes to spend, so the build spends it for you.

## The workflow

```yaml
name: Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo run --release -q -p crabbucket-cli -- build examples/site
      - uses: actions/configure-pages@v5
      - uses: actions/upload-pages-artifact@v4
        with:
          path: examples/site/dist
  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

Then, once:

```sh
gh api -X POST repos/OWNER/REPO/pages -f build_type=workflow
```

## Deploying is a build, and the build is a gate

The workflow runs `crab build`, which fails on a dead link, a missing title
or an unknown layout. A broken site does not reach the upload step, so the
deployment is either the site you meant or no deployment at all.

That is the same sentence as everywhere else in these docs, applied to the
last mile: the build either fails, or the site is correct.
