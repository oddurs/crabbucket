---
id: 48
title: Validate the scaffolded turborust.toml against turborust schema
type: chore
status: done
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

- [x] The schema is vendored or regenerated, with its provenance recorded
- [x] The scaffolded config validates against it
- [x] The hand-written structural assertions are removed
- [x] A schema change fails crabbucket's build rather than a user's

## 2026-09-08

Done with the thorough version, because the cheap one turned out not to be
cheaper. The item offered a `turborust plan` subprocess first and the
vendored schema only if that proved insufficient -- but `turborust plan`
needs turborust installed, which CI does not have, so it skipped every
time and checked nothing.

The vendored schema needs nothing installed and is turborust's own
definition rather than a restatement of it. That distinction is the whole
value: the assertions it replaces encoded turborust's schema in a second
place, which is the shape of problem this project exists to remove.

The question that decided it was whether the schema has
`additionalProperties: false`. Without it, a renamed key would pass
validation as an extra and the check would be theatre. It is false
throughout, so a rename fails. Checked before writing anything.

Three things are validated: both scaffold shapes, and this repository's
own turborust.toml, which is not scaffolded and is the one a contributor
actually runs. A fourth test compares the vendored copy against what
turborust emits today, where it is installed, so drift is a signal rather
than a surprise.

Proved it works by renaming `inputs` to `input` in the template and
watching it fail with `/tasks/site: Additional properties are not allowed
('input' was unexpected)`.

Two dev-dependencies, jsonschema and serde_json, in the CLI crate only.
