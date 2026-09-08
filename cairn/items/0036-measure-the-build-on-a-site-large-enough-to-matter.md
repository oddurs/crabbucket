---
id: 36
title: Measure the build on a site large enough to matter
type: chore
status: done
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: build
---

## Problem

`doc/DESIGN` opens by conceding that speed is not the point — "Hugo builds it
in 30ms and nobody is unhappy". That is an honest position, and it stays
honest only if the numbers are known. Right now nobody knows what a
five-hundred-page build costs.

The risk is specific: syntax highlighting, OpenGraph image generation and
link checking are each capable of turning a fast build into a slow one, and
each was added without a measurement.

## Proposal

Generate a synthetic site — five hundred pages, realistic prose and code
density, a deep section tree — and measure the whole build and each phase.
Compare against Hugo and Zola on the same input, honestly, and publish the
result whatever it says.

Then set a budget and defend it in CI: a build of the synthetic site over
some threshold fails. A benchmark nobody runs is a benchmark that stops being
true within a month.

If the honest answer is that crabbucket is slower than Zola, write that on
the design page. The claim was never that it is the fastest; the claim is
that it is correct, and a wrong claim about speed would undermine the one
that matters.

## Acceptance criteria

- [x] A generator for the synthetic site, committed
- [x] Whole-build and per-phase timings
- [x] Compared against Hugo and Zola on identical input
- [x] A CI budget that fails on regression
- [x] Results published on the site, favourable or not

## 2026-09-08

Done. bench/ generates the same 521 pages three ways and times each; results published on the site's Speed page and cited in doc/DESIGN.

521 pages, best of three, M4 Pro, release: Hugo 145ms, crabbucket 250ms, Zola 314ms. Hugo is about twice as fast; that is written down rather than argued with.

Report gained timings: read 195ms (79%), write 43ms (17%), render 7ms (3%), check 2ms (1%). The typed parts are effectively free. The cost is syntect: the same site with code blocks removed builds in 94ms, so highlighting 2000 blocks is ~150ms, 60% of the build.

The three named risks, measured: highlighting is the whole story; link checking is 2ms and not a risk at all; social cards are, taking a build from 340ms cold to 1.06s and back to 340ms with the cache.

CI budget: 500 pages, 2000ms ceiling, deliberately loose so it fails on a real regression rather than on runner noise.
