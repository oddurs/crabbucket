---
id: 65
title: Redirects, as files a static host will serve
type: feature
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: content
---

## Problem

A page cannot be renamed without breaking every link anybody saved. The build
will tell the author about links *inside* the site, which is most of the
value, and can do nothing about the rest.

## Proposal

Frontmatter on the page that exists now, naming the routes that used to point
at it:

```toml
+++
title = "Routes and links"
aliases = ["routing", "docs/routing"]
+++
```

Each alias is written as a directory containing a meta-refresh page with a
canonical link, which is what a static host can do without configuration.
Zola and Hugo both spell this `aliases` and there is no reason to invent a
different word.

The parts that matter more than the feature:

- An alias joins the route set, so an alias that collides with a real page
  fails the build naming both. Silently shadowing a page with a redirect
  would be a new way to lose one.
- An alias is not a route for link-checking *purposes*: a link written inside
  the site to an alias should fail, because internal links have no excuse.
  That distinction needs a test, since it is the opposite of what the naive
  implementation does.
- Aliases are excluded from the sitemap and the search index, and carry
  `<meta name="robots" content="noindex">`.

## Acceptance criteria

- [ ] `aliases` in frontmatter; one directory per alias with a canonical link
- [ ] An alias colliding with a real route fails the build naming both
- [ ] An internal link to an alias still fails the build
- [ ] Aliases absent from the sitemap and the search index
