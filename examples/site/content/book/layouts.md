+++
title = "Layouts"
description = "Why a layout is a type, what that buys, and what it costs."
layout = "docs"
order = 4
+++

# 4. Layouts

A page says which layout renders it:

```md
+++
title = "Installing"
layout = "docs"
+++
```

In most generators that string is a filename. If it is wrong, you get a
missing-template error at render time, or — worse, and more common — a
different template than you meant, because the lookup fell through to a
default.

Here it is a variant of an enum the design system defines:

```rust
/// The layouts Ferrite offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Prose in one column.
    #[default]
    Page,
    /// Prose with the rest of its section listed beside it.
    #[serde(alias = "landing")]
    Docs,
}
```

So `layout = "dcos"` is a deserialization failure, with the file, the line,
and the list of what exists:

```
crab: error: content/docs/install.md:3: unknown layout `dcos'
  expected one of: page, docs
```

And in the design system, `match page.meta.layout` is exhaustive. Add a
variant and the compiler names every place that has to decide what to do with
it. There is no default branch quietly rendering the wrong thing.

## The associated type

`Layout` is not a type crabbucket defines. It is an associated type on the
`Theme` trait:

```rust sketch
pub trait Theme {
    type Layout: DeserializeOwned + Default;
    type Extra: DeserializeOwned + Default;

    fn render(&self, page: &Page<'_, Self>) -> String where Self: Sized;
    fn stylesheet(&self) -> String;
}
```

Which means the set of layouts a site can use is decided by the design system
it depends on, and the page's frontmatter is checked against *that* set. Not
a global list, not a directory of templates — the type your dependency
exports.

That is the whole seam between a site and its house style, and everything in
[chapters 5](../your-own-design-system/) and [6](../a-fleet-of-sites/) hangs
off it.

## `#[default]`, and why it matters

A page with no `layout` gets the `#[default]` variant. That is what lets
somebody write a page with two lines of frontmatter and have it work.

## `serde(alias)`, and why it matters more

Look again at `Docs`:

```rust
    /// Prose with the rest of its section listed beside it.
    #[serde(alias = "landing")]
    Docs,
```

Ferrite has no `landing` layout. It answers to the name anyway, because a
site written for a design system that *does* have one should still build.

This is the portability contract and it is the part nobody tells you about. A
page's `layout` names something in *its* design system. Swap the design
system and every name has to still mean something, or "change the theme"
means "rewrite the site". One line of `serde(alias)` per name you do not
make a distinction for, and content moves.

It is also how you retire a layout without a breaking change, which is
[chapter 6](../a-fleet-of-sites/) again.

## Using it

The layout is just a value the design system matches on:

```rust
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
```

`page.section("docs")` is every page under `docs/`, in `order`, with the
current one marked. `page.neighbours("docs")` is the two either side, in the
same ordering — deliberately the same, because a reader who follows "next"
should visit the section in the order the sidebar showed them, and two
orderings that can disagree eventually will.

## Next

You have used a design system. [Chapter 5](../your-own-design-system/) is
writing one.
