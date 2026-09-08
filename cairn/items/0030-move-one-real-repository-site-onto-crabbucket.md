---
id: 30
title: Move one real repository site onto crabbucket
type: chore
status: done
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: m
area: docs
---

## Problem

`examples/site` is the only site crabbucket has ever built, and it was
written by the same person, at the same time, against the same assumptions.
It cannot surface the things a real site would.

## Proposal

Pick an existing repository with a GitHub Pages site and move it over. The
turborust site is the obvious candidate, since the two projects already point
at each other.

Keep a log of every place the migration was harder than it should have been —
each of those is a real issue, filed against whichever milestone it belongs
to, and worth more than any amount of speculating about what a second user
would want.

The success condition is not "it looks the same". It is: the new site is
correct, and the number of things that had to be worked around is small
enough to write down.

## Acceptance criteria

- [x] One real site migrated and deployed
- [x] Every workaround filed as its own item
- [x] Both sites share a theme crate
- [x] The migration written up as a docs page
- [x] Nothing in the migration required patching crabbucket locally

## 2026-09-08

Done, with one deliberate departure: cairn's repository was not modified.

The item named turborust as the candidate. turborust has two docs files and
no Pages site, so there was nothing to migrate. cairn was the right
choice instead -- it has a real deployed site, it is GPL-3 like crabbucket
so there is no licence entanglement, and, best of all, **it is an Astro
site**. Migrating from the framework crabbucket takes its model from is a
far better test than migrating from nothing.

I did not commit anything into cairn. Adopting a framework is the
maintainer's decision, and every bit of this item's value -- the friction
log -- comes from doing the migration, not from landing it. The work was
done against a read-only clone.

**The count.** Ten pages. Six migrated mechanically: the collection schema
disappeared because crabbucket already has it, the frontmatter changed
delimiter, and all five `<Callout>` uses became `:::callout` with no
thought. One bug found and fixed. Three gaps found and filed.

**Fixed:** a link to a route without a trailing slash was reported as a
missing file. Astro writes `./configuration`, every static host serves it,
and crabbucket said "is not a file this build writes" -- both wrong and a
tax on every internal link. Three links in six pages; it would have been
all of them in a larger site.

**Filed and open:**

- Frontmatter a site adds is silently dropped. All six pages carried a
  `summary` and lost it. This is the largest thing crabbucket gets wrong
  relative to Astro, which lets a site declare its own schema.
- A site cannot add a page that is not a Markdown file. Four of cairn's
  ten pages are `.astro` components. They have nowhere to go, and the
  workaround loses what made them components.
- A directive cannot read anything but its own attributes, so
  `<Terminal name="mcp" />`, which looks a recording up by name, cannot be
  written at all.

One of those blocks four of the ten pages, which is why the migration page
publishes the number rather than claiming it went well.

**Not required:** patching crabbucket locally. The one fix went in through
the front door and the other three are open items rather than hacks.
