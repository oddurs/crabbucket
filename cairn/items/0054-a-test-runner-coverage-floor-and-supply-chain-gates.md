---
id: 54
title: A test runner, coverage floor and supply-chain gates
type: chore
status: done
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: ci
---

## Problem

There are 320 tests and no framework around them. Three specific holes:

- **Doctests never run.** `cargo test --workspace --all-targets` excludes them,
  and that is what both `make test` and CI use. Every `///` example in the
  public API is unverified.
- **Nobody knows what is covered.** There is no number, so there is no way to
  notice it falling.
- **Nothing checks the dependencies.** This is a GPL-3.0-or-later project about
  to publish seven crates; a GPL-incompatible transitive licence or a security
  advisory would be found by a user, not by CI.

And there is no written answer to "where do I put a new test?", which is how a
suite drifts into whatever each contributor felt like at the time.

## Proposal

The runner: `cargo-nextest`, with a committed `.config/nextest.toml` — a
`default` profile for a laptop and a `ci` profile with a per-test timeout and
JUnit output. Doctests run separately, because nextest does not run them, and
both are behind one `make test`.

The floor: `cargo-llvm-cov` in CI with a threshold that fails the build. Set it
just below wherever the suite actually is, so it ratchets rather than blocks.

The gates: `cargo-deny` for licences, advisories, bans and sources, with the
GPL compatibility rule written down rather than assumed. `cargo-semver-checks`
on pull requests against the merge base, since `doc/STABILITY` makes promises
about the public API and nothing currently holds them.

Then a `HACKING` section naming the layers and saying which one a new test
belongs in.

## Acceptance criteria

- [x] Doctests run, in `make test` and in CI
- [x] `cargo-nextest` with a committed configuration, and a `ci` profile
- [x] A coverage number, reported in CI, with a floor that fails
- [x] `cargo-deny` configured and passing, licences included
- [x] `cargo-semver-checks` on pull requests
- [x] `make` targets for each, and the layers written down in `HACKING`

## 2026-09-08

Done. make test now runs the doctests as well; make cover, make audit and make api are the three new gates and each is its own CI job.

The runner is cargo-nextest with .config/nextest.toml -- a process per test, so there is a per-test timeout, and a ci profile with no retries and JUnit output surfaced in the run summary. It is not a requirement: without it the Makefile says so and falls back to cargo test.

Coverage, with bench/ excluded as a measuring tool rather than the product: 93.3% of regions, 94.3% of lines. The floor is 93 and it ratchets.

cargo-deny found something on its first run, which is the whole argument for having it. syntect's default-fancy feature pulls yaml-load, for reading .sublime-syntax files at runtime, which nothing here does -- and that pulls yaml-rust, unmaintained with RUSTSEC-2024-0320. Trimming syntect to default-syntaxes + html + regex-fancy removes yaml-rust, plist and serde_json outright. Three advisories remain (bincode, rustybuzz, ttf-parser), all unmaintained rather than vulnerable, none with a safe upgrade, each ignored by id with the reason beside it. unmaintained = "all" rather than the default, so a new one is a decision somebody makes.

cargo-semver-checks runs on pull requests against the merge base, since there is no published version to compare against yet. It answers the question a reviewer actually has.

One list of seven published crates now lives in the Makefile, and packaging.rs asserts the release workflow and this test agree with it -- after the workflow failed last week because a publish = false crate was in a list about publishing.
