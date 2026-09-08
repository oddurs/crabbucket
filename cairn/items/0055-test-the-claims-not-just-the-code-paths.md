---
id: 55
title: Test the claims, not just the code paths
type: chore
status: done
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: build
---

## Problem

The 320 tests are example-based: a fixture goes in, an assertion looks at what
came out. That is the right first layer and it has found real bugs, but it only
ever checks the cases somebody thought of, and this project makes claims that
are stronger than "these fourteen inputs work".

Four of them are written down and untested:

- **"Every spelling of a base path normalises to the same thing."** Four
  spellings are tested. There are infinitely many.
- **"A build produces the same output on every machine."** `doc/DESIGN` and the
  deploying page both say it. Nothing builds twice and compares.
- **"The build either fails, or the site is correct."** Nothing checks that the
  HTML and the feeds it emits are even well-formed.
- **The parsers take arbitrary text.** Frontmatter, Markdown, directive
  attributes and link resolution all run on whatever is in a content file, and
  the CRLF bug got through code review on three platforms.

And the tests themselves are unmeasured. A test that asserts nothing passes
just as green as one that asserts everything.

## Proposal

**Properties, not examples.** `proptest` over the invariants above: base-path
normalisation is idempotent and total, `Url` never emits a doubled slash, route
derivation round-trips, `Style` expansion never leaves a `&`, the frontmatter
split is insensitive to line endings, and none of the parsers panic on
arbitrary input.

**Snapshots.** `insta` over a rendered fixture site, per design system, so a
change to a theme arrives as a reviewable diff of the HTML and the stylesheet
rather than as a `contains()` that still passes.

**Determinism.** Build the same site twice into two directories and compare
every byte.

**Validity.** Parse the emitted HTML, sitemap, RSS and Atom, and fail if any of
them is not well-formed.

**Fuzzing.** `cargo-fuzz` targets for the four parsers, seeded from the fixture
sites. On a schedule, not on every pull request. Anything it finds becomes a
unit test.

**Mutation testing.** `cargo-mutants`, to find the assertions that are not
there. Scheduled, because it is slow, and configured so the report is short
enough to act on.

## Acceptance criteria

- [x] Property tests for base paths, URLs, routes, styles and line endings
- [x] No parser panics on arbitrary input
- [x] Snapshot tests of a rendered site, for both design systems
- [x] A test that builds twice and compares every byte
- [x] The emitted HTML, sitemap and feeds are parsed and checked
- [x] Fuzz targets with a seed corpus, run on a schedule
- [x] Mutation testing configured, with the surviving mutants triaged

## 2026-09-08

Done. 320 tests -> 358, coverage 94.3% regions / 95.4% lines, and the suite now holds the claims rather than a list of examples.

Writing the property tests found three real defects, all reachable from site.toml and all in the one type whose entire job is that they cannot happen:

- Url::asset could emit a doubled slash. A feed declared over the collection "notes//archive" would have put one into every feed URL on the site.
- normalize_base could too, for base = "/my//repo/".
- Trimming spaces and then slashes left the space in "repo /" behind.

Url now squeezes runs of slashes, which is the right place: it is the only spot where a URL is assembled. Each has a named regression test beside the property that found it.

Mutation testing was the other productive one. 490 mutants, 381 caught, 55 missed on the first run. The most useful finding: emptying Url's Render implementation broke no test, meaning nothing tested the mechanism by which every link in every page gets the base path. Also unkilled were the whole of closes_fence (five mutants -- the stateful part of the directive scanner, which decides whether a ::: inside a code block is a directive or text), the serde default for base, the content-directory filter, SiteIndex::contains, PageMeta::label, and the four Theme trait defaults -- where a mutant returning Some(vec![]) from og_image would have made every site without social cards start writing empty PNGs.

All of those now have tests. Re-running mutants over theme.rs, config.rs and url.rs: 13 missed -> 2, and both survivors are equivalent mutants (the default already IS None / Default::default(), so the mutation changes nothing).

The remaining ~40 survivors elsewhere are mostly accessors -- is_empty, into_iter, data_files, Debug bodies -- plus tree() and relative_asset(). Worth another pass; not worth blocking this one. The scheduled job reports them.

Also here: a determinism test that builds the same site into two directories and compares every byte; tag-balance and XML checks over what the build writes; snapshots of a page and the stylesheet for both design systems; and four cargo-fuzz targets with a committed seed corpus, which ran 133,000 executions without a crash.

One incidental fix: guide.rs was counting [dev-dependencies] as the design system's dependencies, so adding insta broke a test about what the walkthrough claims.
