+++
title = "Routing"
layout = "docs"
order = 3
+++

# Routing

## Routes come from paths

| File | Route | URL under `base = "/repo/"` |
|---|---|---|
| `content/index.md` | *(root)* | `/repo/` |
| `content/about.md` | `about` | `/repo/about/` |
| `content/docs/index.md` | `docs` | `/repo/docs/` |
| `content/docs/routing.md` | `docs/routing` | `/repo/docs/routing/` |

Any `index.md` takes the route of its directory. Every route is written as a
directory containing an `index.html`, so every URL ends in a slash and needs
no server rewrite rules.

## The base path is applied in exactly one place

A GitHub Pages *project* site is served from `/repo/`. A user site or a custom
domain is served from `/`. Getting that wrong is the most common way one of
these sites ships broken — a stylesheet that 404s, a nav that leaves the site.

`base` is recorded once in `site.toml` and applied by one type:

```rust
pub struct Url(String);

impl Url {
    pub fn new(config: &Config, path: &str) -> Self { … }   // a page
    pub fn asset(config: &Config, path: &str) -> Self { … } // a file
}
```

Nothing else in the framework, and nothing in a site's own components, ever
concatenates a base path. `Url` implements `maud::Render`, so a component
interpolates one directly and cannot accidentally interpolate a `String`
instead:

```rust
a href=(Url::new(config, "docs/routing")) { "Routing" }
```

Every spelling of a base path normalises to the same thing — `repo`,
`/repo`, `repo/` and `/repo/` are all `/repo/` — so the config file cannot
express the bug either.

## Link checking is a build gate

Content is Markdown, so a link written in a page cannot be typed the way a
link written in a component can. There is no compiler in a `.md` file to
catch it.

So the build catches it instead. After every page is rendered — and only
then, because a link is dead only relative to the finished set of routes —
every internal `href` and `src` is resolved and looked up:

```
crab: 2 dead internal links:
  content/docs/routing.md: layout/ -> /docs/layout/ is not a route
  content/index.md: intro/#setup -> /docs/intro/#setup has no such heading
```

The build fails. It is not a warning and there is no flag to turn it off,
because a link checker you can turn off is a link checker that is off.

What gets checked:

- **Relative links** resolve against the page's own URL, so `content/` from
  `/repo/docs/` is `/repo/docs/content/`. Prefer these; they survive a change
  of `base`.
- **Site-absolute links** must start with the base path.
- **Links ending in a slash** are routes, checked against the route set.
- **Links not ending in a slash** are assets, checked against `static/` plus
  the files the build itself emits.
- **Fragments** are checked against the target page's [heading
  ids](../content/). This is the one that earns its keep: headings get
  reworded far more often than pages get renamed. A bare `#fragment` is
  checked against the page it was written in.

What does not get checked: anything with a scheme, anything protocol-relative,
`mailto:`, `tel:`, `data:`, and bare `#fragments`. External liveness is not a
property of this build, and a generator that phones out to the network to
decide whether it succeeded is a generator that fails on an aeroplane.

A bare `#` is left alone: it is the conventional spelling of "no
destination", not a broken link.

## The error page

`content/404.md` becomes `dist/404.html` — a file rather than a directory,
because that is the name a static host looks for. It is a page but not a
route: nothing is required to link to it, and it does not appear in
navigation.

Its own links are still checked, and one extra rule applies to it. The error
page is served in place of *any* missing path, so a relative link on it has no
directory to resolve against — `docs/` would mean something different
depending on where the reader was. The build refuses to write one:

```
crab: 1 dead internal link:
  content/404.md: docs/ -> docs/ is relative, and the error page is served
    from every path; write it as a site-absolute link
```

## The other half: routes as a type

Content has no compiler, so link checking is the best it can have. A Rust
component *does* have one, and should not settle for less.

A site that is a crate generates a `Route` enum from `content/` in its
`build.rs`:

```rust
// build.rs
let found = crabbucket_routes::scan(&content)?;
print!("{}", crabbucket_routes::watch(&content, &found));

let routes = found.into_iter().map(|(route, _)| route).collect::<Vec<_>>();
fs::write(out.join("routes.rs"), crabbucket_routes::generate(&routes)?)?;
```

```rust
pub mod routes {
    include!(concat!(env!("OUT_DIR"), "/routes.rs"));
}

html! { a href=(Url::new(config, Route::DocsRouting.path())) { "The route table" } }
```

Rename `content/docs/routing.md` and the variant stops existing:

```
error[E0599]: no variant, associated function, or constant named
`DocsRouting` found for enum `Route` in the current scope
```

Three things worth knowing:

**Two routes that would share a variant name fail the build**, naming both.
`docs/getting-started` and `docs/getting/started` both camel-case to
`DocsGettingStarted`, and picking one silently would mean a link that compiles
and goes to the wrong page.

**The content directory is watched as well as the files in it.** Cargo only
notices a *new* file through the directory, and a build script watching the
files alone misses the page you just added — the classic build-script bug.

**`crabbucket-routes` has no dependencies**, and compiles in about a quarter of
a second. That matters because a build dependency is compiled before anything
else; using `crabbucket` itself would have dragged a Markdown parser and a
syntax highlighter into that phase to walk a directory.

The two mechanisms stay separate and neither replaces the other. Link checking
covers content, which cannot be typed; the enum covers components, which can.

See `examples/site-crate` for a working one.
