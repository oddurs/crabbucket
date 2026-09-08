---
id: 53
title: A directive cannot read anything but its own attributes
type: feature
status: done
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

- [x] A directive can read site-supplied data and the configuration
- [x] The data is deserialized into the site's own types
- [x] The existing simple registration form still works unchanged
- [x] What a handler cannot see, and why, is documented

## 2026-09-08

Done, and it turned out to be two things rather than one.

The item was about a directive seeing more than its attributes. Building
the demonstration showed the other half: a *site* could not register a
directive at all -- only its design system could. cairn's `<Terminal>`
belongs to cairn, not to a theme, so the context alone would have had no
user. `Options::directives` closes that, and a name the design system
already uses is an error rather than an override.

`add_with` is fallible where `add` is not, deliberately: the reason to want
a context is almost always to look something up, and a lookup that misses
should stop the build at the directive's line rather than render nothing.

The ordering constraint the item predicted held exactly. Directives run
while content is being loaded, so there is no site index and a handler
cannot ask about other pages -- and that ordering is what makes the route
table and link checking possible. The configuration and `data/*.toml` exist
before any of it, so those are what a handler gets. Documented as a
limitation rather than left to be discovered.

One API change fell out: `build_with` takes `Options` by value now, because
`Directives` holds boxed closures and cannot be borrowed out of a shared
reference. `Options` lost its `PartialEq`, and the CLI's `Command` with it,
so three assertions became `matches!`.

One thing I noticed and did not act on: `Page<'_, Self>` from 0051 blocks a
site from wrapping a theme to add behaviour, because `Page<'_, Wrapper>`
and `Page<'_, Inner>` are different types even when their Layout and Extra
match. `Options::directives` serves the case that prompted this, so the
wrapping question can wait for something that actually needs it.
