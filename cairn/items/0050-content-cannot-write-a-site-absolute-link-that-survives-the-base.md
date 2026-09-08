---
id: 50
title: Content cannot write a site-absolute link that survives the base
type: bug
status: backlog
milestone: v0.3
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: routing
---

## What happens

The error page must use site-absolute links, because it is served in place of
any path and a relative link has no directory to resolve against. So
`examples/site/content/404.md` writes `/crabbucket/docs/`.

That hard-codes the base path into content. Build the same site with
`--base /elsewhere/` and the link resolves to `/crabbucket/docs/`, which is
outside the base, is therefore not checked, and is wrong in the output.

Found while building the example site with a second design system, which was
passing a base override at the time.

## What should happen

`base` is "recorded once and applied in one place". That is true of everything
the framework generates and false of anything content writes absolutely.

## Proposal

Content needs a way to say "the site root" without naming it. Options, cheapest
first:

- A leading `~/`, resolved against the base by the link rewriter, the way a
  shell resolves a home directory. One character, no new syntax to parse,
  and unambiguous because `~` is not otherwise meaningful at the start of a
  path.
- Rewrite every root-relative link in content against the base. Simple, but it
  makes it impossible to link to something else on the same domain.
- A `:::link` directive. Correct and heavy for what it buys.

Whichever it is, the link checker has to understand it, and `--base` has to
move it.

## Acceptance criteria

- [ ] Content can write a site-root-relative link without naming the base
- [ ] `--base` moves it, and link checking follows
- [ ] The example site's 404 page uses it
- [ ] Building the example site under a base override passes
- [ ] Documented on the routing page beside the error-page rule
