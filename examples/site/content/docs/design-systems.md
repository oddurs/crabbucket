+++
title = "Design systems"
layout = "docs"
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
that every site in a fleet depends on:

```toml
[dependencies]
my-house-style = { git = "https://github.com/me/house-style" }
```

```rust
crabbucket::build(Path::new("."), &my_house_style::Theme)?;
```

Restyling every site you own is then a version bump, which is the actual
reason to want first-class design systems on a collection of small sites
rather than on one big one.

## What is not built yet

Per-component scoped CSS. Today a theme returns one stylesheet and the whole
thing ships. For sites of this size that is a few kilobytes and the right
trade; the thing worth building is not tree-shaking but **collision
avoidance**, so that components from two different crates cannot fight over a
class name. See [Design](../design/).
