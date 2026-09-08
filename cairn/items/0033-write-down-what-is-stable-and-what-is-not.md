---
id: 33
title: Write down what is stable and what is not
type: docs
status: done
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

- [x] `doc/STABILITY` covering both lists
- [x] MSRV policy stated
- [x] Deprecation path stated
- [x] `#[non_exhaustive]` decided and applied
- [x] Referenced from `README` and `HACKING`

## 2026-09-08

Done. `doc/STABILITY`, seven sections and 136 lines.

The section worth having is 3, "The awkward one": a design system's class
names are its public surface and nothing in this project checks them,
because a site's stylesheet is not something the build sees. That is the
one place the central claim does not hold, and a stability document that
listed only the things it gets right would be worth less than none.

Section 5 makes the decision the item said could not wait. Error, Reason,
Config, Options and Report are non_exhaustive: all five have gained
members in every release so far, and deciding this after 1.0 would itself
be the breaking change. PageMeta deliberately is not -- it is deserialized
rather than constructed, and its shape is the frontmatter schema, which
section 1 already covers.

That decision had a cost worth recording: non_exhaustive forbids struct
expressions from other crates entirely, `..Default::default()` included.
So Options gained a builder, which reads better than the literal did, and
Config::blank() replaced Config::for_tests with fields assigned rather
than listed.

MSRV: raising it is a minor version. A major version for every toolchain
bump would make the major number about Rust rather than about crabbucket.

Seven tests hold what a test can hold. Most of the document is a
commitment rather than a property -- nothing can stop somebody renaming a
method -- but the three surface rules and the section 5 decisions are
checkable, and now checked.
