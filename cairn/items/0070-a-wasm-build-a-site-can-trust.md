---
id: 70
title: A wasm build a site can trust
type: feature
status: backlog
milestone: v2.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: build
---

## Problem

[[0037]] needs wasm in the build, and that is the part most likely to make the
project worse. Every current answer drags in a toolchain: `wasm-pack` wants
npm, `trunk` wants to own the whole build, `wasm-bindgen` needs its CLI
version pinned to its crate version or the output is silently wrong.

crabbucket's build is `cargo build` and nothing else. A site's deploy is a
`cargo run` in a GitHub Action. Whatever this becomes must not change either of
those sentences.

## Proposal

The constraint first, because it decides the design: **a site with no islands
compiles no wasm, needs no extra tool, and its build command does not change.**
Islands are opt-in per design system, and the cost lands only on the design
system that opts in.

Then, for the ones that do:

- **One target per design system, not per island.** A page with three islands
  from one design system loads one wasm module. Per-island modules sound
  tidier and mean three fetches and three copies of shared code.
- **`wasm-bindgen` with the CLI version derived from the locked crate
  version**, so it cannot drift. A mismatch must fail the build with the two
  versions in the message; it is a famously bad silent failure.
- **The wasm build is a separate cargo invocation the design system's own build
  produces**, not something `crabbucket` shells out to during a site build. A
  site consuming a design system gets a pre-built, versioned `.wasm` the same
  way it gets a pre-built stylesheet — which keeps a site's build a
  `cargo run`, and means a site does not need the wasm target installed at all.

That last point is the one worth getting right. It puts the toolchain cost on
the person writing the design system, once, and none of it on the twelve sites
using it. It also means the `.wasm` is content-addressed and cached like any
other asset from [[0062]].

**A size budget, in CI, from the first commit.** The project already has a
build-time budget that fails; a wasm bundle without one grows without anybody
deciding to. The number to beat is the thing being replaced: the existing
router and search client together are about 5KB gzipped, and Pagefind's wasm
search client is around 10KB. An island runtime that costs 200KB has lost the
argument regardless of how well typed it is.

## Acceptance criteria

- [ ] A design system with no islands: no wasm, no new tool, same build command
- [ ] One module per design system, content-addressed, cached
- [ ] `wasm-bindgen` CLI and crate versions cannot drift silently
- [ ] A site's build stays `cargo run` and needs no wasm target
- [ ] A gzipped size budget enforced in CI, and the number published
- [ ] The reproducibility test extends to the wasm artefact
