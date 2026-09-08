---
id: 47
title: Nothing checks the turborust config crabbucket ships
type: bug
status: backlog
milestone: v0.2
labels:
- coupling
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: cli
---

## What happens

`crab new` writes a `turborust.toml` into every scaffolded site, and this
repository has one of its own. Both assert a schema that belongs to another
project, and nothing verifies either.

turborust is under active development and is not a dependency, so a schema
change reaches crabbucket's users as a broken scaffold, not as a build error.
It has already drifted once: `doc/DESIGN` section 8 and the dev-loop page list
turborust's commands, and `connect` and `reloads` were added after they were
written.

Coupling points, all unverified:

- `crates/crabbucket-cli/src/templates/turborust.toml` — shipped to users
- `turborust.toml` — this repository's own
- `examples/site/content/docs/dev-loop.md` — quotes the config and commands
- `doc/DESIGN` section 8 — quotes the config
- `README`, `HACKING` — mention the workflow

## What should happen

A change to turborust's schema should fail crabbucket's build, not a user's.

## Proposal

The cheap version, worth doing first: a test that runs `turborust plan` against
the scaffolded config and asserts it parses, skipped with a message when
turborust is not installed. Cheap, and it catches the case that actually hurts.

The thorough version, only if the cheap one proves insufficient: turborust
exposes its config type from a library crate, and crabbucket parses the
template against it in a test. That makes it a real dependency, which is a
larger commitment than the problem currently justifies.

Note the licences point in the other direction: turborust is MIT OR Apache-2.0
and crabbucket is GPL-3.0-or-later, so code may move from turborust to
crabbucket but not back.

## Acceptance criteria

- [ ] A test parses the scaffolded turborust.toml with turborust itself
- [ ] The test skips with a message, rather than failing, when it is absent
- [ ] The documented command list is checked or dated
- [ ] Run in CI on a machine that has turborust
