+++
title = "Your own design system"
layout = "docs"
order = 7
+++

# Your own design system

[Design systems](../design-systems/) explains the mechanism. This is the
walkthrough: a whole one, from an empty directory to twelve sites sharing it.

Every snippet here is from `crabbucket-theme-plain`, a real crate in this
repository that CI compiles and that builds this very site in its own test
suite. If a snippet drifts from the code, a test fails.

:::callout{kind = "note", title = "Why a crate"}
A design system is a crate you depend on. That is the whole point: restyling
twelve repository sites is then a version bump rather than twelve pull
requests.
:::

## 1. The crate

```toml
[package]
name = "crabbucket-theme-plain"

[dependencies]
crabbucket = "0.1"
maud = "0.27"
serde = { version = "1", features = ["derive"] }

[build-dependencies]
crabbucket-tokens = "0.1"
```

Four dependencies. `crabbucket` for the `Theme` trait, `maud` for the markup,
`serde` because layouts and component props are deserialized, and
`crabbucket-tokens` at build time only.

## 2. The tokens

A design system owns its palette. Reaching into crabbucket's would make every
theme a fork of the default one.

```toml
# design/tokens.toml
[color]
paper  = "#fdfdfb"
ink    = "#1b1b19"
faded  = "#5c5c56"
rule   = "#d9d7cf"
accent = "#7a1f1f"

[color.dark]
paper  = "#16161a"
ink    = "#e6e4de"
```

The base palette is whichever scheme you design in; the sub-table is the other
one. Plain is light-first because it is a document, and documents are light.

```rust
// build.rs
let module = generate(&source, Scheme::Light)?;
fs::write(out.join("tokens.rs"), module)?;
```

```rust
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}
```

That is the whole build script. It was forty lines in the first design system,
and became a crate the moment a second one needed the same forty.

## 3. The layouts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    #[default]
    #[serde(alias = "docs", alias = "landing")]
    Page,
}
```

An enum rather than a string, so `layout = "dcos"` fails the build listing
what exists. See [Layouts](../layouts/).

The aliases are the **portability contract**, and they are the part nobody
tells you about. A page's `layout` names something in *its* design system. A
site written for a theme with three layouts will not build against one with a
different three — unless the new one says which distinctions it does not make.

Plain has one layout and answers to three names. One line, and a site can move.

## 4. The shell, and one component

```rust
pub const NS: &str = "pl";

pub const PAGE: Style = Style::new(NS, "page", include_str!("styles/page.css"));
pub const PROSE: Style = Style::new(NS, "prose", include_str!("styles/prose.css"));
```

```css
/* styles/page.css */
.& {
  background: var(--color-paper);
  color: var(--color-ink);
  font-family: var(--font-body);
}

.&__masthead { border-bottom: 1px solid var(--color-rule); }
```

`&` is the component's class root, so `.&__masthead` becomes
`.pl-page__masthead`. Two design systems can both have a `page` and neither
wins.

Then the trait:

```rust
impl Theme for Plain {
    type Layout = Layout;

    fn render(&self, page: &Page<'_, Layout>) -> String {
        document(page.config, page.meta, &page.nav(), page.feeds, prose(page.html))
            .into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([PAGE, PROSE]);
        format!("{}\n{}", tok::CSS, sheet.render())
    }
}
```

Three methods. Everything else — the router, search, social cards, directives
— is a trait default that says "not this design system", and a site asking for
one is told once and still builds.

## 5. Answering to the content

A page's `:::callout` names a component. A design system that cannot answer to
the name cannot render the page, however different it chooses to look.

Plain implements the same vocabulary and makes different choices:

```rust
directives.add("callout", |props: Callout, body| {
    html! {
        aside class=(/* … */) {
            @if let Some(title) = &props.title { (label(title)) }
            (body)
        }
    }
});
```

A callout here is an indented note rather than a coloured box. Tabs are every
panel one after another, because a design system with no scripting cannot hide
one behind another and pretending otherwise would leave half the page
unreadable.

The props are the same types by name and shape, deliberately. Content written
against one design system's attributes has to keep working against another's,
or "swap the theme" means "rewrite the site".

## 6. Using it

```sh
crab new my-project --theme https://github.com/me/house-style
```

That scaffolds a site *crate*, because a site that depends on a design system
has to call the build itself:

```rust
use house_style::Standard as Design;

fn main() -> ExitCode {
    match crabbucket::build(Path::new("."), &Design) {
        Ok(report) => { println!("{report}"); ExitCode::SUCCESS }
        Err(err) => { eprintln!("{err}"); ExitCode::FAILURE }
    }
}
```

## 7. Twelve sites

```toml
house-style = { git = "https://github.com/me/house-style", tag = "v2.1.0" }
```

Restyling all of them is a version bump. That is the claim this whole design
exists to make good on, and it holds exactly as far as the next section says
it does.

## The parts that are annoying

Nobody writes these down, so here they are.

### Adding a layout

Free. A site that does not use it is unaffected, because the enum grew a
variant nothing names.

### Removing a layout

Breaking. Any page saying `layout = "gallery"` fails the build with `unknown
variant`, naming the file and the line. That is the good failure — the site
does not build rather than rendering wrong — but it is a breaking change and
belongs in a major version.

Keep the name as an alias for whatever replaced it if you can:

```rust
#[serde(alias = "gallery")]
Grid,
```

### Adding a token

Free.

### Removing or renaming a token

Breaking for *your own components*, immediately and at compile time:

```
error[E0425]: cannot find value `ACCENT` in module `tok::color`
```

Free for sites, which never name tokens — they name components. This is the
asymmetry worth understanding: tokens are internal to a design system, and
components and layouts are its public surface.

### Renaming a component's class

Breaking for anyone who wrote CSS against it. Nothing catches that, because a
site's own stylesheet is not something the build checks. It is the one place
this design has no guarantee to offer, so treat class names as public.

### Migrating a site across a breaking change

1. Bump the dependency in one site, not twelve.
2. Build. The failures are layout names and component names, each with a file
   and a line.
3. Fix them, or add aliases to the theme and no site needs fixing at all.
4. Then bump the rest.

The build being a gate is what makes this bearable: you find out at step 2
rather than from a reader.

## What a second design system is for

`crabbucket-theme-plain` exists because a trait with one implementation is a
guess about what the abstraction needs. Writing it changed five things in
crabbucket, and has since caught three bugs that no single-theme test could —
including one in the page you are reading.

If you write one, the same will happen. That is the point of it.
