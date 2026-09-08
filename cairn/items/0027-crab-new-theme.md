---
id: 27
title: '`crab new --theme`'
type: feature
status: done
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

- [x] `--theme NAME` writes a crates.io dependency
- [x] `--theme URL` writes a git dependency
- [x] The scaffolded site crate builds with no edits
- [x] Without `--theme`, the existing content-only scaffold is unchanged
- [x] Both shapes documented

## 2026-09-08

Done early, and not for the reason it was filed. An external review of
the project found that `crab new` scaffolds a turborust.toml that is wrong
for the custom-theme path the docs describe: it writes `cmd = "crab build"`
with content-only inputs, so a site that became a crate rebuilt nothing when
its Rust changed. Editing the stylesheet did nothing until they fixed the
config by hand.

So there are now two scaffold shapes rather than one plus a footnote. A
content directory gets `crab build` and content inputs. `--theme` gets a
Cargo.toml, a src/main.rs that calls crabbucket::build, inputs that include
`src/**` and `Cargo.toml`, and a workflow that runs `cargo run --release`
instead of installing crab.

One thing turborust's source settled: `inputs` is derived from `cargo` only
when `inputs` is empty. So `cargo = "name"` and an explicit content list are
mutually exclusive, and taking the derivation would silently stop watching
content. The crate template lists both explicitly and says why in a comment.

Did not wait for 0025, the second theme. The scaffold's correctness does not
depend on a second theme existing, and the finding was live.

Tested: both shapes parse, the crate shape watches src, and -- behind
CRABBUCKET_SLOW_TESTS, which CI sets -- the generated crate is compiled
against the working copy and run, so `--theme` cannot emit code that does not
build.
