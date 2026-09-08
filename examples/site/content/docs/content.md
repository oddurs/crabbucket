+++
title = "Content"
layout = "docs"
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
heading attributes enabled. Code fences keep their language tag, which is
what syntax highlighting will hang off when it lands.

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
