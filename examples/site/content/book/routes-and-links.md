+++
title = "Routes and links"
description = "Where a file ends up, what the base path really costs, and why a dead link stops the build."
layout = "docs"
order = 3
+++

# 3. Routes and links

## Where a file ends up

| File | Route | URL under `base = "/ferrite/"` |
|---|---|---|
| `content/index.md` | *(root)* | `/ferrite/` |
| `content/about.md` | `about` | `/ferrite/about/` |
| `content/docs/index.md` | `docs` | `/ferrite/docs/` |
| `content/docs/install.md` | `docs/install` | `/ferrite/docs/install/` |

Any `index.md` takes the route of its directory. Every route is written as a
directory containing an `index.html`, so every URL ends in a slash and no
server needs rewrite rules. Two files that would claim the same route fail
the build naming both, because one of them silently winning is precisely the
class of failure this framework refuses.

## The base path, which is the thing that goes wrong

A GitHub Pages *project* site is served from `/repository/`. A user site or a
custom domain is served from `/`. Getting this wrong is the single most
common way a small site ships broken: the stylesheet 404s, the navigation
leaves the site, and it all worked locally.

`base` is written once, in `site.toml`, and no page ever mentions it. One
type applies it:

```rust sketch
pub struct Url(String);

impl Url {
    pub fn new(config: &Config, path: &str) -> Self;   // a page
    pub fn asset(config: &Config, path: &str) -> Self; // a file
}
```

Nothing else in the framework — and nothing in your own components —
concatenates a base path. `Url` implements `maud::Render`, so a component
interpolates one directly and cannot pass a bare `String` by accident.

Every spelling normalises to the same value: `ferrite`, `/ferrite`,
`ferrite/` and `/ferrite/` are all `/ferrite/`. The config file cannot
express the bug either.

And because it is an input rather than content, the same site can build for
two places at once — a project page and a pull-request preview — which is
[chapter 6](../a-fleet-of-sites/)'s problem.

## Saying "the site root"

Prefer relative links. `../install/` survives a change of `base` because it
never mentions one.

Sometimes you cannot. The 404 page is served in place of *any* path, so it
has no directory to be relative to. Writing `/ferrite/docs/` by hand
hard-codes the base into a page, and that fails *silently* — a link outside
the base is not this build's business, so nothing complains when the site
moves.

So `~/` means the site root, whatever the base is:

```md
The [documentation](~/docs/) is probably where you were going.
```

```
/ferrite/   →  /ferrite/docs/
/           →  /docs/
/preview/   →  /preview/docs/
```

It is resolved on the finished page, so it works in content and in components
alike, and it is checked afterwards like any other link.

## Why a dead link stops the build

Content is Markdown. There is no compiler in a `.md` file, so a link written
in a page cannot be typed the way one written in a component can.

So the build checks them instead — after every page has rendered, because a
link is dead only relative to the *finished* set of routes:

```
crab: 2 dead internal links:
  content/docs/install.md: layout/ -> /docs/layout/ is not a route
  content/index.md: install/#cargo -> /docs/install/#cargo has no such heading
```

The build fails. It is not a warning, and there is no flag, because a link
checker you can turn off is a link checker that is off.

The rules, in one list:

- **Relative links** resolve against the page's own URL.
- **Site-absolute links** must start with the base path.
- **Links ending in a slash** are routes.
- **Links not ending in a slash** are assets — checked against `static/`
  *plus* every file the build itself emits, so a link to a generated feed or
  a social card resolves.
- **Fragments** are checked against the target page's heading ids.

The fragment one earns its keep more than any of the others. Headings get
reworded constantly; pages get renamed rarely.

Nothing with a scheme is checked, nor `mailto:`, `tel:`, `data:`, nor
protocol-relative URLs. External liveness is not a property of this build,
and a generator that phones out to the network to decide whether it succeeded
is a generator that fails on an aeroplane. A bare `#` is left alone: it is
the conventional spelling of "no destination".

## The cost

On a 521-page site, checking 3,507 links takes two milliseconds. It is
[measured](~/docs/speed/), and it is the cheapest thing the build does.

## Next

Pages exist and point at each other correctly. [Chapter
4](../layouts/) is about how they get rendered.
