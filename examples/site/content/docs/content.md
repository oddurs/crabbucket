+++
title = "Content"
layout = "docs"
order = 2
+++

# Content

A page is a Markdown file with a `+++` TOML frontmatter block at the top.
Writing one needs no Rust.

```markdown
+++
title = "Getting started"
layout = "docs"
nav_order = 1
+++

Install it with `cargo install`.
```

## The frontmatter every page has

| Field | Type | Meaning |
|---|---|---|
| `title` | string | Required. Used in `<title>` and as the default nav label. |
| `description` | string | Optional. Falls back to the site description. |
| `layout` | the theme's layout type | Which layout renders this page. See [Layouts](../layouts/). |
| `nav_order` | number | Put this page in the primary navigation, at this position. |
| `nav_label` | string | A shorter label for navigation than the title. |
| `draft` | bool | Skip the page entirely. |

A field the schema does not know about is an error, not a silently ignored
key. Frontmatter that does not fit fails the build with the file and the line
inside it, because `toml` reports both and the error type carries the path:

```
crab: content/docs/routing.md: TOML parse error at line 2, column 1
  |
2 | ttile = "Routing"
  | ^^^^^
missing field `title`
```

## Markdown

CommonMark, with tables, footnotes, strikethrough, smart punctuation and
heading attributes enabled.

### Headings are linkable

Every heading gets an id derived from its text — `## Getting started` becomes
`getting-started` — and a permalink anchor that appears on hover. Write
`{#custom-id}` after a heading to choose the id yourself; someone holding a
link to it wins over the slug. Repeated headings on one page get `-2`, `-3`.

The ids are what [fragment links are checked against](../routing/).

### Code is highlighted at build time

A fenced block with a language tag is highlighted while the site is built,
into spans carrying scope classes. The colours come from
[`design/tokens.toml`](../design-systems/) like every other colour.

Nothing is shipped to the browser to make code coloured. Shipping forty
kilobytes of JavaScript to colour text that was already static when it was
written is exactly the trade this project exists to refuse.

A fence with no language, or with one nothing is known about, renders as
plain code. A build that failed over a language tag would be absurd.

## Typed collections

The frontmatter above is the *built-in* schema — what `crab build` needs in
order to render a page with no Rust anywhere. A site that wants more declares
its own type:

```rust
use crabbucket::Collection;
use serde::Deserialize;

#[derive(Deserialize)]
struct DocMeta {
    title: String,
    order: u32,
    since: String,
}

let docs: Collection<DocMeta> = Collection::load(Path::new("content/docs"))?;

for entry in &docs {
    println!("{} ({})", entry.meta.title, entry.route);
}
```

A page missing `order` fails to load, and the error names the file and the
line. Astro validates this shape at runtime with a schema object; here it is
the collection's type parameter, so there is no schema to keep in sync with
the type — the type *is* the schema.

Sorting, grouping and filtering are ordinary Rust, because a collection is a
slice of typed entries rather than a value inside a template language:

```rust
let mut entries: Vec<_> = docs.entries().iter().collect();
entries.sort_by_key(|entry| entry.meta.order);
```

## What is not built yet

Directives — `:::callout{kind="warn"}` in Markdown resolving to a typed
component function — are described in [Design](../design/) and are not
implemented. Until they are, Markdown is Markdown.
