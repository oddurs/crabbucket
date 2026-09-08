+++
title = "Design systems"
+++

# Design systems

`design/tokens.toml` is the source of truth.

```toml
[color]
surface = "#0b0d10"
accent  = "#e8623c"
```

One build script generates both halves from that file: the Rust constants a
component refers to, and the CSS custom properties the browser resolves.
They cannot drift apart, because a single generator emits both.

```rust
use crabbucket_ui::tok;

html! {
    div style={ "color: " (tok::color::ACCENT) } { "…" }
}
```

Rename `accent` and every use site stops compiling. That is the whole
mechanism.

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

Typed props, HTML checked when the crate compiles, no template language and
no runtime between the two. It compiles to a `write!` chain, so rendering
costs approximately nothing.

## A theme is a crate

`crabbucket-ui` is the default. The interesting case is a private crate
shared by every site in a fleet: restyling all of them is a version bump.
