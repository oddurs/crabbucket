---
id: 32
title: Publish the crates to crates.io
type: chore
status: backlog
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: ci
---

## Problem

Installation is `cargo install --git`, which pins nobody to anything, gives
no version history, and makes a dependency on `crabbucket-ui` awkward for
anyone building a theme.

## Proposal

Publish `crabbucket`, `crabbucket-ui` and `crabbucket-cli`, in that order,
since the later ones depend on the earlier.

Before the first publish, and only once, because names and initial choices
are hard to take back:

- Confirm the names are free and take them.
- Check the metadata: description, keywords, categories, `readme`,
  `repository`, and the licence field. `readme = "README"` has no extension,
  so verify crates.io renders it rather than assuming it will.
- Decide what ships in the package. `examples/site` and `design/` are needed
  for the build script; `target/` and CI configuration are not. Use `include`
  rather than `exclude`, so a new directory does not get published by
  accident.
- Dry-run every crate and inspect the resulting archive by hand.

Then a release checklist in `HACKING` and a tag-triggered workflow, so the
second release is not also a research project.

## Acceptance criteria

- [ ] All three crates published and installable with plain `cargo install`
- [ ] Package contents verified by inspecting the archive
- [ ] crates.io renders the extensionless README
- [ ] Release checklist in `HACKING`
- [ ] Tag-triggered publish workflow, with a dry run on pull requests
