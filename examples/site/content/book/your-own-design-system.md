+++
title = "Your own design system"
description = "A whole one, from an empty crate to a stylesheet, in about a hundred lines."
layout = "docs"
order = 5
+++

# 5. Your own design system

A design system is a crate you depend on. Not a theme directory, not a folder
of templates you copied in — a crate, with a version number.

That is the whole point, and [chapter 6](../a-fleet-of-sites/) is what it
buys. This chapter is building one.

We are going to build **Ferrite**: light, serif, two layouts, one component,
no JavaScript. It is about a hundred and fifty lines and it renders a whole
site. Everything below is a real crate in crabbucket's repository, and a test
asserts that what you are reading is what that crate contains.

## 1. The crate

```toml
[package]
name = "ferrite"
edition = "2024"

[dependencies]
crabbucket = "0.1"
maud = "0.27"
serde = { version = "1", features = ["derive"] }

[build-dependencies]
crabbucket-tokens = "0.1"
```

Four dependencies. `crabbucket` for the `Theme` trait, `maud` for markup that
the compiler checks, `serde` because layouts and props are deserialized, and
`crabbucket-tokens` at build time only.

## 2. The tokens

A design system owns its palette. Reaching into somebody else's would make
every theme a fork of that one.

```toml
# design/tokens.toml
[color]
paper  = "#fbf9f6"
ink    = "#22201d"
faded  = "#6b665e"
rule   = "#e0dbd2"
accent = "#a8431f"

[color.dark]
paper  = "#1a1815"
ink    = "#ece8e1"

[space]
line = "1.5rem"
half = "0.75rem"

[font]
body = "Charter, 'Bitstream Charter', Georgia, serif"

[measure]
column = "36rem"
```

The base table is whichever scheme you design in; the sub-table is the other
one. Ferrite is light-first because it is a document, and documents are
light.

One file, two outputs. The build script generates a Rust module:

```rust
fn main() {
    let tokens = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("design/tokens.toml");

    println!("cargo::rerun-if-changed={}", tokens.display());
    println!("cargo::rerun-if-changed=build.rs");

    let source =
        fs::read_to_string(&tokens).unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let module = generate(&source, Scheme::Light)
        .unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("no out dir")).join("tokens.rs");
    fs::write(&out, module).unwrap_or_else(|err| panic!("{}: {err}", out.display()));
}
```

```rust
/// Design tokens, generated from this crate's own `design/tokens.toml`.
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}
```

That is the whole build script. It was forty lines in crabbucket's first
design system and became a crate the moment a second one needed the same
forty.

What you get is `tok::color::ACCENT` in Rust and `--color-accent` in CSS,
from one source, so the two cannot drift apart. A renamed token is a compile
error in your own components, immediately.

The generator also checks contrast: a foreground and a background that fail
WCAG AA fail the build, in both schemes. A palette that is unreadable is not
a matter of taste.

## 3. Styles, and the `&`

```rust
/// The namespace every class in this design system carries.
pub const NS: &str = "fe";
```

```rust
/// The page shell.
pub const PAGE: Style = Style::new(NS, "page", include_str!("styles/page.css"));

/// Everything Markdown produces.
pub const PROSE: Style = Style::new(NS, "prose", include_str!("styles/prose.css"));

/// An aside beside the prose.
pub const NOTE: Style = Style::new(NS, "note", include_str!("styles/note.css"));
```

Each `Style` is a CSS file that knows its own class root:

```css
/* styles/page.css */
.& {
  background: var(--color-paper);
  color: var(--color-ink);
  font-family: var(--font-body);
}

.&__masthead {
  border-bottom: 1px solid var(--color-rule);
  display: flex;
  gap: var(--space-line);
  padding: var(--space-line);
}
```

`&` is replaced by the component's class, so `.&__masthead` becomes
`.fe-page__masthead`. Two design systems can both have a `page` and neither
wins. The stylesheet is assembled from the styles a theme actually uses, so a
component you do not render ships no CSS.

## 4. The trait

Three methods, and one of them is optional.

```rust
impl Theme for Ferrite {
    type Layout = Layout;
    type Extra = Extra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        let body = match page.meta.layout {
            Layout::Page => prose(page.html),
            Layout::Docs => {
                let section = page.route.split('/').next().unwrap_or("");
                docs(&page.section(section), prose(page.html))
            }
        };

        document(page.config, page.meta, &page.nav(), body).into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([PAGE, PROSE, NOTE]);
        format!("{}\n{}", tok::CSS, sheet.render())
    }

    fn directives(&self) -> Directives {
        let mut directives = Directives::new();

        directives.add("callout", |props: Callout, body| {
            html! {
                aside class=(NOTE.class()) {
                    @if let Some(title) = &props.title {
                        strong class=(NOTE.element("title")) { (title) }
                    }
                    (body)
                }
            }
        });

        directives
    }
}
```

That is a complete design system. `render` gets a `Page` — the config, the
typed frontmatter, the route, the rendered HTML, the headings, the site index
— and returns a string. `stylesheet` returns the CSS. `directives` registers
components; the default is none.

Everything else is a trait default that says "not this design system": no
router, no search client, no social cards. A site that asks for one anyway
gets a warning naming what it asked for, and still builds. That matters more
than it sounds: the alternative is a site configured for search, shipping a
search box, wired to a file that was never written.

## 5. The pieces

The functions the `match` calls are ordinary Rust returning `maud::Markup`:

```rust
/// Renders a body of Markdown-derived HTML.
///
/// The HTML comes from the site's own content, which is trusted, so it is
/// emitted unescaped.
pub fn prose(html_fragment: &str) -> Markup {
    html! { div class=(PROSE.class()) { (PreEscaped(html_fragment)) } }
}
```

```rust
/// A list of links, with the current one marked.
pub fn links(items: &[NavItem]) -> Markup {
    html! {
        @for item in items {
            a href=(item.href) aria-current=[item.current.then_some("page")] { (item.label) }
        }
    }
}
```

`maud` is compile-checked HTML. An unclosed tag is a syntax error, not a
malformed page, and interpolated values are escaped unless you say otherwise
— which `prose` does say, deliberately and with a comment, because the HTML
there came from the site's own Markdown.

## 6. Answering to the content

A page's `:::callout` names a component. A design system that cannot answer
to the name cannot render the page — however differently it chooses to look.

Ferrite's callout is the `directives` method above: an aside with an optional
title. The default design system draws a coloured box with an icon. Same
name, same props, different result, and the same content builds against
both.

Keep the prop types the same by name and shape as whatever your content
already uses. That is what makes "swap the design system" mean something
other than "rewrite the site".

## 7. Using it

```sh
crab new ferrite-docs --theme https://github.com/me/ferrite
```

That scaffolds a site *crate*, because a site with its own design system has
to call the build itself. [Chapter 6](../a-fleet-of-sites/) is the whole of
what that file contains and why it is worth it.

## What writing one actually teaches you

crabbucket's second design system changed three things in the framework
itself, and each of those was the real deliverable:

1. `router_js` and `search_js` returned `String`. A design system without a
   router had no way to say so, and a site asking for one got an empty file.
   They return `Option<String>` now.
2. The token generator lived inside the first design system's build script,
   so the second one's first draft was a forty-line copy of it. It is a crate.
3. That generator assumed a dark palette with light overrides, because that
   was the only palette it had ever seen.

A trait with one implementation is a guess about what the abstraction needs.
If you write a second one, you will find something too — that is not a
warning, it is the reason to do it.

## Next

[Chapter 6](../a-fleet-of-sites/) is why any of this was worth doing.
