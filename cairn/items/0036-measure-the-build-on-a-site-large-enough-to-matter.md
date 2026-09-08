---
id: 36
title: Measure the build on a site large enough to matter
type: chore
status: backlog
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

- [ ] A generator for the synthetic site, committed
- [ ] Whole-build and per-phase timings
- [ ] Compared against Hugo and Zola on identical input
- [ ] A CI budget that fails on regression
- [ ] Results published on the site, favourable or not
