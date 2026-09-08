---
id: 3
title: '`crab new` scaffolds a working site'
type: feature
status: backlog
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: cli
---

## Problem

Starting a site means knowing that `site.toml` needs a `base`, that content
lives in `content/`, that GitHub Pages needs `.nojekyll` and a workflow, and
that the dev loop is a `turborust.toml`. That is a page of documentation
standing between someone and their first build, for something a command can
do in a second.

## Proposal

    crab new my-site [--base /my-site/] [--router]

writes:

    my-site/
      site.toml            title from the directory name, base from --base
      content/index.md     a landing page with real frontmatter
      content/docs/index.md
      static/.gitkeep
      turborust.toml       tasks.site + services.web, ports free
      .github/workflows/pages.yml
      .gitignore           /dist

and then prints the two commands that follow: `crab build` and
`turborust up`.

Refuse to write into a non-empty directory unless `--force`. Scaffolding over
someone's files is the kind of thing a tool gets one chance at.

`--base` should default to `/NAME/`, because a project site is the common
case and a user site is the one worth typing a flag for.

## Acceptance criteria

- [ ] `crab new x && cd x && crab build` succeeds with no edits
- [ ] The scaffolded site passes link checking
- [ ] Non-empty target directory is refused without `--force`
- [ ] The generated `turborust.toml` runs under `turborust up`
- [ ] Templates live in the CLI crate, not in a runtime data directory
