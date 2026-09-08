---
id: 27
title: '`crab new --theme`'
type: feature
status: backlog
milestone: v0.3
depends_on:
- 3
- 25
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: cli
---

## Problem

Once a house style is a crate, every new site starts by editing the
scaffolded `Cargo.toml` to point at it. That is the fleet workflow, and it is
manual.

## Proposal

    crab new my-site --theme my-house-style
    crab new my-site --theme https://github.com/me/house-style

A name resolves as a crates.io dependency; a URL as a git one. The scaffold
then produces a site crate rather than a bare content directory, since a
custom theme means the site has to call `build` itself.

Two scaffold shapes, then, and `crab new` picks by whether `--theme` was
given. Document both in the man page.

## Acceptance criteria

- [ ] `--theme NAME` writes a crates.io dependency
- [ ] `--theme URL` writes a git dependency
- [ ] The scaffolded site crate builds with no edits
- [ ] Without `--theme`, the existing content-only scaffold is unchanged
- [ ] Both shapes documented
