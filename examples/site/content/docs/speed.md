+++
title = "Speed"
description = "What the build actually costs, measured against Hugo and Zola on the same pages."
layout = "docs"
order = 13
+++

# Speed

[Design](/docs/design/) says the advantage of a typed build is not speed. That
is only an honest thing to say if the numbers are known, so here they are,
including the one that is not flattering.

## The measurement

`bench/` in the repository generates the same synthetic site three ways --
crabbucket, Hugo and Zola -- and times each of them. The pages are
byte-for-byte identical: the frontmatter is the intersection of what the three
accept, and every page has the same prose, the same four headings, the same
four Rust code blocks and the same table.

```sh
cargo run --release -p crabbucket-bench -- generate 500 /tmp/bench
cargo run --release -p crabbucket-bench -- time /tmp/bench
```

521 pages, best of three runs, on an Apple M4 Pro with crabbucket built in
release mode, Hugo 0.165.0 and Zola 0.23.4.

| Generator | Best | Doing |
|---|---:|---|
| Hugo | 145ms | highlighting |
| crabbucket | 250ms | highlighting, link checking |
| Zola | 314ms | highlighting |

**Hugo is roughly twice as fast as crabbucket, and crabbucket is roughly
twenty percent faster than Zola.** That is the result; it is not going to be
dressed up. Hugo has had a decade of people caring about exactly this number,
and it shows.

Two things the table does not say. crabbucket resolves and checks every
internal link and every fragment as part of the build -- 3,507 of them here --
which the other two do not do at all. And all three are, at this size, faster
than the time it takes to switch to a browser window.

## Where the time goes

The build reports its own passes, so this is measured rather than guessed:

| Pass | | Share |
|---|---:|---:|
| read and parse | 195ms | 79% |
| write | 43ms | 17% |
| render | 7ms | 3% |
| check links | 2ms | 1% |

The interesting number is the small one. Handing 521 pages to the design
system costs seven milliseconds, and checking every link on them costs two.
The typed parts of the build -- the parts this framework exists for -- are
free. Nearly all of the cost is upstream of them.

Within that upstream cost, the culprit is syntax highlighting. The same site
with its code blocks removed builds in **94ms**, with the read pass falling
from 195ms to 48ms. Highlighting two thousand code blocks with `syntect`
therefore costs about 150ms: sixty percent of the whole build.

That is a fine trade. Highlighting happens once, at build time, so no reader
ever downloads a highlighter -- and a docs site that spends most of its build
colouring code is a docs site with a lot of code in it.

## Social cards

The numbers above are for a site with no `url`, which is a site with no
[social cards](/docs/social-cards/) — a card is only ever referenced from an
absolute URL, so a site without one is not asked for any.

Turning them on is the most expensive thing you can do to a crabbucket build.
Drawing 521 cards takes the whole thing from 340ms to **1.06s**, measured with
`crab build` so process startup is in both. That is about 1.4ms per card, which
is a reasonable price for rasterising an SVG and a fair reason for the cache:
cards are written to `.crabbucket/og/`, and the second build is back to 340ms.

So the honest advice is to keep `.crabbucket/` between builds. On a laptop it
is free. In CI, cache it or accept an extra second — for a site of this size,
once.

## The budget

CI builds the same 500-page site on every pull request and fails if it takes
longer than two seconds. The ceiling is deliberately loose: a shared runner's
timings vary by more than any change worth making, and a budget that fails on
noise is a budget everybody learns to pass with a rerun. It is there to catch
a change that makes the build several times slower, and it will let everything
smaller through.

If you want a tighter number, run the bench on your own machine, where nothing
else is competing for the cores.
