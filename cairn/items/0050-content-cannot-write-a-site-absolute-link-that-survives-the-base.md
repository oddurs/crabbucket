---
id: 50
title: Content cannot write a site-absolute link that survives the base
type: bug
status: done
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

- [x] Content can write a site-root-relative link without naming the base
- [x] `--base` moves it, and link checking follows
- [x] The example site's 404 page uses it
- [x] Building the example site under a base override passes
- [x] Documented on the routing page beside the error-page rule

## 2026-09-08

Done, with the first of the three options: a leading `~/`, resolved
against the base.

Resolved on the finished page rather than in the Markdown pass, which was
not the plan and is better: it works in content and in components alike,
and nothing downstream has to know the convention exists. By the time the
link checker sees it, it is an ordinary absolute link and is checked like
one -- `~/gone/` fails exactly as `/repo/gone/` would.

Only inside `href` and `src`. These docs contain `~/Code/crabbucket` in
prose, and rewriting that would have been worse than the problem being
solved. There is a test named after it.

The example site's error page now uses it, and the seam test in
crabbucket-theme-plain builds the example site with `--base /` again --
the override it had to avoid when this was filed.
