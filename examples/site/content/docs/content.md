+++
title = "Content"
+++

# Content

A page is Markdown with a `+++` TOML frontmatter block. Writing one needs
no Rust.

```markdown
+++
title = "Getting started"
nav_order = 1
+++

Install it with `cargo install`.
```

## Typed collections

A collection is declared with the type its frontmatter deserializes into.

```rust
#[derive(Deserialize)]
struct DocMeta {
    title: String,
    order: u32,
}

let docs: Collection<DocMeta> = Collection::load(Path::new("content/docs"))?;
```

A page missing `order` fails the build, and the error names the file and
the line inside it. Astro validates this shape at runtime; here it is the
collection's type parameter.

Sorting, grouping and filtering are ordinary Rust, because a collection is
a slice of typed entries rather than a value in a template language.

## Routes

Routes come from paths. `content/docs/content.md` is `/docs/content/`, and
any `index.md` takes the route of its directory. Every route is a directory
holding an `index.html`, so every URL ends in a slash and works on a plain
static host with no rewrite rules.
