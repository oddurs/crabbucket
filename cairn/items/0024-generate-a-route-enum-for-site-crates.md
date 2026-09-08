---
id: 24
title: Generate a Route enum for site crates
type: feature
status: backlog
milestone: v0.3
labels:
- design
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: routing
---

## Problem

`doc/DESIGN` section 3 describes two halves of link safety. Link checking
built the content half. This is the other one: a link written in a Rust
component is still a string, and a renamed page breaks it at build time
rather than at compile time.

For a site whose components are Rust — which is every site with a custom
design system — that is the difference between finding out when you save the
file and finding out when you run the build.

## Proposal

A site crate's `build.rs` calls into crabbucket to scan `content/` and emit
an enum into `OUT_DIR`:

    pub enum Route { Index, Docs, DocsRouting, … }
    pub fn path(route: Route) -> &'static str { … }

used as:

    a href=(Url::route(config, Route::DocsRouting)) { "Routing" }

Open questions to settle while building it, not before:

- **Crate weight.** `crabbucket` pulls in maud and pulldown-cmark; as a
  build-dependency that is compiled twice. Splitting the route scanner into a
  dependency-free `crabbucket-routes` crate is the obvious fix, and should be
  the answer only if it measurably matters.
- **Staleness.** `cargo::rerun-if-changed=content` covers edits, but adding a
  file to a watched *directory* is the case build scripts classically miss.
  Test it explicitly.
- **Naming.** `docs/getting-started.md` to `DocsGettingStarted` collides with
  `docs/getting/started.md`. Detect collisions and fail the build rather than
  silently picking one.

This does not replace link checking. Content still has no compiler, so both
mechanisms stay, each covering the half it can.

## Acceptance criteria

- [ ] A site crate's `build.rs` generates the enum from `content/`
- [ ] A renamed page fails compilation at every use site
- [ ] Adding a file causes a rebuild
- [ ] Route name collisions fail the build with both paths named
- [ ] `examples/site` gains a Rust half that uses it
- [ ] Build-dependency compile cost measured and recorded on this item
