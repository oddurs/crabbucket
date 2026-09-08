+++
title = "Components"
layout = "docs"
order = 5
+++

# Components

A Markdown page reaches a typed component through a **directive**:

```markdown
:::callout{kind = "warn", title = "Careful"}
crabbucket needs Rust 1.85 or newer.
:::
```

Attributes are a TOML inline table, so quoting and escaping are TOML's problem
rather than a bespoke parser's, and the syntax is the one the rest of the
project already uses. They deserialize into the component's own props type
before the component runs — so a misspelled attribute fails the build:

```
crab: content/docs/components.md:7:1: `callout`: unknown field `knid`
  |
7 | :::callout{knid = "warn"}
  | ^^^^^^^^^^^^^^^^^^^^^^^^^
```

That is MDX's expressiveness without MDX's compiler.

## callout

:::callout{kind = "note", title = "This is a note"}
Neutral information, the kind a reader can skip.
:::

:::callout{kind = "warn", title = "This is a warning"}
Something the reader can get wrong. `kind` defaults to `note`, and `title` is
optional.
:::

## cards

For index pages. A card is a real `<a>`, so its `href` is checked like every
other link — a card pointing at a page that does not exist fails the build.

:::cards
:::card{title = "Routing", href = "../routing/"}
Routes, the base path, and link checking.
:::
:::card{title = "Layouts", href = "../layouts/"}
Why a layout is a type and not a string.
:::
:::card{title = "Design systems", href = "../design-systems/"}
Tokens, components, and themes as crates.
:::
:::

## tabs

:::tabs{group = "install"}
:::tab{label = "cargo"}
```sh
cargo install --git https://github.com/oddurs/crabbucket crabbucket-cli
```
:::
:::tab{label = "from source"}
```sh
git clone https://github.com/oddurs/crabbucket
cd crabbucket && make install
```
:::
:::

These tabs use no JavaScript. Each tab is a radio input, its label, and its
panel — three siblings, so CSS alone decides which panel shows and the
component needs to know nothing about its neighbours. With scripting disabled
they still work; that is the whole design rather than a fallback.

Two tab groups sharing a `group` name move together, which is what you want
for "macOS / Linux / Windows" repeated down a page. Remembering the choice
across pages is the one part that needs script, so it happens only when the
site has opted into the [router](../design/).

## steps

:::steps
:::step{title = "Scaffold"}
`crab new my-project` writes a site that builds with no edits.
:::
:::step{title = "Write"}
Markdown in `content/`. A file's path is its route.
:::
:::step{title = "Deploy"}
Push. The workflow builds and publishes, and refuses to publish a site with a
dead link in it.
:::
:::

The numbers come from a CSS counter, so reordering steps cannot leave them
wrong.

## Adding your own

A design system registers directives by name and by type together:

```rust
fn directives(&self) -> Directives {
    let mut directives = Directives::new();
    directives.add("callout", |props: Callout, body| callout(&props, body));
    directives
}
```

`add` takes the props type as a parameter, so the handler never sees an
attribute it did not ask for and never has to check one it did. A name nothing
is registered under fails the build listing the names that are.

## Directives the site owns

A component usually belongs to a design system. Not always: a `terminal` that
reads recordings the site generates belongs to the *site*, and no design
system should have to know about it.

```rust
let options = Options {
    directives: my_directives(),
    ..Options::default()
};
```

A name the design system already uses is an error. A site silently replacing a
component would change every page that uses it without a word.

## What a directive can see

`add` gives a handler its attributes and its body, which is all most of them
need. `add_with` gives it a context as well:

```rust
directives.add_with("terminal", |props: Terminal, _, context| {
    let recordings: Recordings = context.data("recordings")?;

    let recording = recordings
        .by_name
        .get(&props.name)
        .ok_or_else(|| format!("there is no recording called `{}`", props.name))?;

    Ok(html! { pre { code { "$ " (recording.command) } } })
});
```

`context.data("recordings")` reads `data/recordings.toml` into the site's own
type. A missing file, a file that does not fit the type, and a lookup that
misses all fail the build at the directive's line:

```
crab: content/docs/index.md:24:1: `terminal`: there is no recording called `nope`
   |
24 | :::terminal{name = "nope"}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^
```

`add_with` is the fallible form for exactly that reason: the point of wanting
a context is usually to look something up, and a lookup that misses should
stop the build rather than render nothing.

:::callout{kind = "note", title = "What a directive cannot see"}
Other pages. Directives run while content is being loaded, so there is no site
index yet — and that ordering is what makes the route table and link checking
possible at all. The configuration and the data files exist before any of it,
so those are what a handler gets.
:::

## Directives inside code fences

Everything on this page is a real directive, and every example of one *in a
code block* is not — the scanner tracks fences, which is why it works on lines
rather than on the Markdown parser's event stream.
