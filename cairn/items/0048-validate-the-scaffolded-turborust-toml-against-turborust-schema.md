---
id: 48
title: Validate the scaffolded turborust.toml against turborust schema
type: chore
status: backlog
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: cli
---

## Problem

The scaffolded `turborust.toml` is checked by a hand-written structural test
that asserts the keys crabbucket relies on. That catches our own typos, but it
encodes turborust's schema by hand in a second place — which is the shape of
problem crabbucket exists to remove.

## Proposal

`turborust schema` emits a JSON Schema for `turborust.toml`. Vendor it, or
regenerate it in CI, and validate the scaffolded config against it.

Strictly better than the structural test: it is turborust's own definition, it
does not need turborust installed at test time, and a schema change shows up as
a diff rather than as a silent divergence.

Replaces the hand-written assertions in
`crates/crabbucket-cli/tests/scaffold.rs`.

## Acceptance criteria

- [ ] The schema is vendored or regenerated, with its provenance recorded
- [ ] The scaffolded config validates against it
- [ ] The hand-written structural assertions are removed
- [ ] A schema change fails crabbucket's build rather than a user's
