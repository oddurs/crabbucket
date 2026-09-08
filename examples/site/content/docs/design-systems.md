+++
title = "Design systems"
layout = "docs"
order = 6
+++

# Design systems

The design system is the framework's core, not a plugin bolted to its side.
That is the difference between a site generator you theme and a site generator
you build a house style with.

## One token file, two outputs

`design/tokens.toml` is the source of truth:

```toml
[color]
surface        = "#0b0d10"
surface-raised = "#11151b"
accent         = "#e8623c"

[space]
xs = "4px"
sm = "8px"
md = "16px"

[measure]
prose = "68ch"
```

One build script reads it and emits both halves of every token: the Rust
constant a component refers to, and the CSS custom property the browser
resolves.

```rust
// generated
pub mod color {
    /// `#e8623c` -- also available to CSS as `var(--color-accent)`.
    pub const ACCENT: &str = "var(--color-accent, #e8623c)";
}
```

```css
/* generated */
:root { --color-accent: #e8623c; --space-md: 16px; --measure-prose: 68ch; }
```

They cannot drift apart, because a single generator emits both. Rename
`accent` and every use site stops compiling:

```
error[E0425]: cannot find value `ACCENT` in module `tok::color`
```

Each constant is a `var(--name, fallback)` reference rather than a bare hex
value, so a component still picks up a runtime theme override — a
`prefers-color-scheme` block, a `[data-theme]` attribute — while keeping the
compile-time guarantee.

The rule that makes it hold: **nothing outside the generated token module may
contain a colour or a length.**

## Components are functions

```rust
pub fn callout(kind: Kind, body: Markup) -> Markup {
    html! {
        aside class={ "callout callout--" (kind.slug()) } {
            div."callout__body" { (body) }
        }
    }
}
```

Typed props. HTML checked when the crate compiles — an unclosed tag is a
compile error, not a rendering artefact. No template language, no runtime, and
no parser for anyone to maintain: `maud`'s macro compiles the block down to a
sequence of pushes onto one `String`, so rendering a page costs about what
concatenating it costs.

Props are types too, so a callout cannot be given a kind that does not exist,
and adding a variant to `Kind` tells you every place that needs updating.

## A theme is a crate

`crabbucket-ui` is the default one. The interesting case is a private crate
that every site in a fleet depends on. Restyling every site you own is then a
version bump, which is the actual reason to want first-class design systems on
a collection of small sites rather than on one big one.

A site that uses one is **a crate rather than a content directory**, because
it has to call the build itself and say which design system to use:

```sh
crab new my-project --theme https://github.com/me/house-style
```

```
my-project/
  Cargo.toml       crabbucket, and the theme
  src/main.rs      calls crabbucket::build with the theme
  site.toml
  content/
```

```rust
use house_style::Standard as Design;

fn main() -> ExitCode {
    match crabbucket::build(Path::new("."), &Design) {
        Ok(report) => { println!("{report}"); ExitCode::SUCCESS }
        Err(err) => { eprintln!("{err}"); ExitCode::FAILURE }
    }
}
```

That is the whole program. `Report` implements `Display` and says everything
it has to say — the page and link counts, any skipped drafts, and any warnings
— so the obvious `println!` is also the complete one.

:::callout{kind = "warn", title = "The two shapes need different dev configs"}
A content directory is built by `crab build` and watches `content/`. A crate
is built by `cargo run` and has to watch `src/` too, or editing the stylesheet
rebuilds nothing. `crab new --theme` writes the right one; the difference is
not cosmetic and it is silent when it is wrong.
:::

## Loading your clients

A design system that offers a router or a search client has to load it. The
head is the theme's, so nothing else can put the tag there:

```rust
@if config.router { script defer src=(Url::asset(config, "router.js")) {} }
@if config.search { script defer src=(Url::asset(config, "search.js")) {} }
```

Forgetting one ships a feature that silently does nothing — the file is
written, no page loads it, and the search box sits hidden waiting for a client
that never arrives. So the build says so:

```
crab: warning: search.js was written but no page loads it; the design
system's document template is missing its script tag
```

A design system with no router and no search says nothing at all, and gets
neither the file nor the warning; a site that asks for one anyway is told
once, and still builds.

## Hooking a navigation

The router swaps `<main>` without reloading, so anything a design system
attaches to the DOM has to run again afterwards. It fires an event rather than
calling components by name:

```js
addEventListener('crabbucket:render', () => {
  // Runs on first load and after every client-side navigation.
});
```

The built-in tab memory, contents highlighting and copy buttons all go through
that same event, which is deliberate: if the door rots, the default design
system breaks first. Forking the router to add one function call would
silently cost you prefetching, scroll-spy, tab persistence and copy buttons.

## Writing one

[Your own design system](../your-own-design-system/) is the walkthrough: a
whole design system from an empty directory to twelve sites sharing it,
including the parts that are annoying — what a version bump breaks, and what
it does not.

## Styles live beside components

Each component declares its own styles, with `&` standing in for its class
root:

```rust
pub const CALLOUT: Style = Style::new(NS, "callout", "
    .& { border-left: 3px solid var(--color-accent); }
    .&--warn { border-left-color: var(--color-accent); }
    .&__body > :last-child { margin-bottom: 0; }
");
```

```rust
CALLOUT.class()           // cb-callout
CALLOUT.with("warn")      // cb-callout cb-callout--warn
CALLOUT.element("body")   // cb-callout__body
```

That is BEM with the block name factored out, and the block name namespaced in
exactly one place. The theme composes a `StyleSheet` from the components it
declares, deduplicating by namespace *and* name together — so two design
systems may both have a `card` and neither one wins.

A small fixed set of class names belongs to the framework rather than to any
theme: `heading-anchor`, `code`, and the `tok-` classes the highlighter emits.
They are listed in `crabbucket::style::FRAMEWORK_CLASSES`, so "what a theme
does not own" is checkable rather than remembered.

This is deliberately *not* tree-shaking. At the size these sites run at, a few
kilobytes of unused CSS is optimising the wrong number. The problem worth
solving was collision, and collision is a naming mechanism rather than a build
pipeline.
