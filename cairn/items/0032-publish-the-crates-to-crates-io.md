---
id: 32
title: Publish the crates to crates.io
type: chore
status: doing
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
- [x] Package contents verified by inspecting the archive
- [x] crates.io renders the extensionless README
- [x] Release checklist in `HACKING`
- [x] Tag-triggered publish workflow, with a dry run on pull requests

## 2026-09-08

Everything up to the publish is done; the publish itself is deliberately not, because a name and a version cannot be taken back and that is the maintainer's call to make, not an agent's.

All seven names are free on crates.io, checked today: crabbucket, -cli, -ui, -og, -routes, -tokens, -theme-plain.

Three real defects found by packaging rather than by reading:

1. crabbucket-ui's build.rs read ../../design/tokens.toml, outside the package. The published crate would not have compiled. Moved into the crate, which is where crabbucket-theme-plain already had it — each design system owns its tokens.
2. Several test files read doc/STABILITY and doc/crab.1. tests/ now ships in no crate; a test suite that cannot compile is worse than none.
3. crates.io renders an extensionless readme AS MARKDOWN, not as plain text (verified against crates_io_markdown: no extension takes the Markdown path). The README is GNU plain text with two-space aligned blocks, which CommonMark reflows into the paragraph above. Reindented to four; a test now holds it, along with the backtick pairing.

Also: include lists everywhere (allow-list, not deny-list), keywords and categories on the six that lacked them, and a COPYING copy in each crate with a test that they cannot drift from the top-level one.

cargo package --workspace --locked builds and verifies all seven. Archives inspected by hand; contents listed in the PR.

Left to do, by the maintainer: decide the version — the workspace says 0.1.0 while NEWS has three unreleased sections, and the first published number is permanent — add the CARGO_REGISTRY_TOKEN secret to a crates-io environment, and push the tag. HACKING has the checklist.
