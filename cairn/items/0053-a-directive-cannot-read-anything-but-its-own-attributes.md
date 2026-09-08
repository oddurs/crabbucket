---
id: 53
title: A directive cannot read anything but its own attributes
type: feature
status: backlog
milestone: v1.0
labels:
- migration
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: content
---

## Problem

A directive handler is `Fn(Props, Markup) -> Markup`. It sees its attributes
and its body, and nothing else.

cairn's site has `<Terminal name="mcp" caption="..." />`, which looks a
recording up by name from data the site generates. There is no way to write
that as a crabbucket directive: the handler cannot reach the site's data, the
configuration, or even the page it is on.

The same limit explains why the tabs component had to derive its radio group
from a counter rather than from anything it could ask about.

## Proposal

Give a handler a context: the `Config`, and a site-supplied data map read from
somewhere like `data/*.toml`. Deserialized into the site's own types, in
keeping with everything else here.

    directives.add_with("terminal", |props: Terminal, body, site| {
        let recording = site.data::<Recordings>()?.get(&props.name)?;
        …
    });

Keep the simple form. Most directives are pure functions of their attributes
and should stay that way; this is for the ones that are not.

Note the ordering constraint: directives run while content is being loaded,
before the site index exists, so a handler cannot see other pages. Data and
configuration are available; routes are not, without restructuring the build.

## Acceptance criteria

- [ ] A directive can read site-supplied data and the configuration
- [ ] The data is deserialized into the site's own types
- [ ] The existing simple registration form still works unchanged
- [ ] What a handler cannot see, and why, is documented
