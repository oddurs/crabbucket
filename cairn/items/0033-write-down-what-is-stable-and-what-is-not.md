---
id: 33
title: Write down what is stable and what is not
type: docs
status: backlog
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: s
area: docs
---

## Problem

A 1.0 without a written stability policy is a version number, not a promise.
Users cannot tell which parts they may depend on, and the author cannot tell
which changes require a major version — so in practice every change becomes
a judgement call made in a hurry.

## Proposal

`doc/STABILITY`, short and specific about what is covered:

- The `site.toml` schema, the frontmatter schema, and route derivation.
- The `Theme` trait and everything a theme crate touches.
- `Url`, `Collection`, `Entry`, `PageMeta`, `SiteIndex`, `Error`.
- The CLI: commands, options, output shape, and exit statuses.

And what is not: the generated HTML structure, the class names in the default
theme, the token *values* (though not their names), and anything documented
as unstable.

State the MSRV policy — how far back, and what bumping it costs — and the
deprecation path: a release that warns before a release that removes.

Adding an `Error` variant is a breaking change for anyone matching
exhaustively. Decide now whether the enum is `#[non_exhaustive]`, because
deciding later is itself the breaking change.

## Acceptance criteria

- [ ] `doc/STABILITY` covering both lists
- [ ] MSRV policy stated
- [ ] Deprecation path stated
- [ ] `#[non_exhaustive]` decided and applied
- [ ] Referenced from `README` and `HACKING`
