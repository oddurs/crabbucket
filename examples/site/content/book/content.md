+++
title = "Content"
description = "Markdown, frontmatter as a struct, and the moment a page stops being a string."
layout = "docs"
order = 2
+++

# 2. Content

Ferrite's docs site has three pages so far. This chapter is about filling it
up, and about the one idea that makes the rest of the book work.

## A page

```md
+++
title = "Installing"
description = "Two ways, and which one you want."
nav_order = 2
+++

# Installing

The short version is `cargo add ferrite`.
```

TOML between `+++` fences, then Markdown. One file, one page.

## Frontmatter is a struct

This is the idea. In most generators, frontmatter is a map: you write a key,
the template asks for a key, and if the two ever disagree you find out from a
reader.

Here it is deserialized into a Rust type before the page is rendered, so the
disagreement is a build failure with a file and a line:

```
crab: content/docs/installing.md: TOML parse error at line 2, column 1
  |
2 | ttile = "Installing"
  | ^^^^^
missing field `title`
```

Every page has `title`, and optionally `description`, `layout`, `nav_order`,
`nav_label`, `order`, `date` and `draft`. The [reference](../reference/) has
the table.

Two of those are worth a sentence each. `nav_order` puts a page in the
site-wide navigation at that position; a page without one is not in it.
`order` sorts a page within its own section, which is what the sidebar and
the previous/next links follow — one ordering, so the sidebar and the "next"
button cannot disagree.

## Fields your design system adds

The list above is what the build needs to render *any* page. A design system
can ask for more, and the asking is a type:

```rust
/// The frontmatter Ferrite reads beyond what every page has.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Extra {
    /// A sentence that stands above the prose.
    #[serde(default)]
    pub summary: Option<String>,
}
```

Now a page may carry `summary = "..."`, the design system reads
`page.meta.extra.summary`, and it is an `Option<String>` rather than a lookup
that might miss. Take the `Option` off and a page without one fails to load.

A site cannot invent a field its design system does not read. That is the
same bargain as layouts, and [chapter 6](../a-fleet-of-sites/) is about why
it is a good one.

:::callout{kind = "warn", title = "One hole, stated plainly"}
serde cannot combine `flatten` with `deny_unknown_fields`, so a *misspelled*
key that nothing declares is still ignored silently rather than failing.
`sumary = "..."` disappears. It is the one place this design does not hold,
and a test asserts it so the day it changes is a deliberate one.
:::

## Markdown, and what the build does to it

CommonMark, plus tables, footnotes, strikethrough, smart punctuation and
heading attributes.

Two things happen that are worth knowing about.

**Headings become linkable.** `## Getting started` gets the id
`getting-started` and a permalink anchor. Write `{#custom-id}` to choose it
yourself. These ids are what [chapter 3](../routes-and-links/) checks
fragment links against, and they are the reason a reworded heading does not
quietly break six links.

**Code is highlighted while the site builds.** A fenced block with a language
becomes spans carrying scope classes, coloured by your design system's
tokens. Nothing is sent to the browser to make code coloured — shipping forty
kilobytes of JavaScript to colour text that was already static is exactly the
trade this project exists to refuse. It is also, measurably, [most of the
build](~/docs/speed/).

A fence with no language, or an unknown one, renders as plain code. A build
that failed over a language tag would be absurd.

## Components in prose

A page can use a component without leaving Markdown:

```md
:::callout{kind = "warn", title = "Careful"}
This deletes the index.
:::
```

`callout` names something the design system registered. The attributes are a
TOML inline table, deserialized into that component's props type — so a
misspelled attribute, or one of the wrong type, fails the build with the page
and the line. So does a directive nothing is registered under.

That is the whole reason this exists instead of letting people write raw HTML
into Markdown: raw HTML cannot be wrong, it can only look wrong.

## Drafts and dates

`draft = true` skips a page entirely. It is not written, not linked, not in
the search index; the build says how many it skipped.

`date = 2026-01-14` is a real TOML date, not a string that hopes to be one.
Dates are what [feeds](~/docs/feeds/) are built from, and configuring a feed
over a directory is how a site declares that everything in it is dated — so
an undated page in a dated collection fails the build naming itself.

## Collections, when you want them

Everything above needs no Rust. If your site *is* a crate — which it will be
by [chapter 5](../your-own-design-system/) — you can also load a directory as
a typed collection and treat it as data:

```rust sketch
#[derive(Deserialize)]
struct Release {
    title: String,
    version: String,
    date: toml::value::Datetime,
}

let releases: Collection<Release> = Collection::load(dir, &directives, &context)?;
```

A page missing `version` fails to load, naming the file. Sorting and grouping
are ordinary Rust, because a collection is a slice of typed entries rather
than a value inside a template language.

## Next

You now have pages. [Chapter 3](../routes-and-links/) is where they end up
and how they point at each other.
