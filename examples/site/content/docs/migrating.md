+++
title = "Migrating from Astro"
layout = "docs"
order = 12
+++

# Migrating from Astro

crabbucket takes its model from Astro, so a migration is mostly a rename. This
page is the record of doing one — cairn's documentation site, ten pages, ten
components — and of everything that was harder than it should have been.

Four things were. Three of them are still open, and are named at the bottom.

## What moved without work

**Content collections become content.** Astro:

```ts
const docs = defineCollection({
  loader: glob({ base: "./src/content/docs", pattern: "**/*.{md,mdx}" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    order: z.number(),
  }),
});
```

crabbucket has that schema already, so the collection definition disappears
and the frontmatter changes delimiter:

```markdown
+++
title = "Durability"
description = "What happens on a crash, a merge, or two people writing at once."
order = 4
layout = "docs"
+++
```

**Components become directives.** Astro:

```jsx
import Callout from "../../components/Callout.astro";

<Callout tone="warn">
  Rename is atomic within a filesystem.
</Callout>
```

crabbucket:

```markdown
:::callout{kind = "warn"}
Rename is atomic within a filesystem.
:::
```

The import line goes away, because a directive is registered by the design
system rather than imported by the page. All five callouts in the real site
converted mechanically.

**Routes are the same.** `src/content/docs/durability.mdx` is `/docs/durability/`
in both. `index` is the directory in both.

## What was harder than it should have been

### Links without a trailing slash — fixed

Astro writes `[the schema](./configuration)`. crabbucket read that as an asset
and reported *"is not a file this build writes"* for something that is a
route, and which every static host serves with a redirect.

Three links in six pages, and it would have been every internal link in a
larger site. The checker now treats a slashless link that names a route as the
route it names. No workaround is needed and the migration needs no edits to
its links at all.

### Frontmatter the site adds is dropped — fixed

Every one of the six pages carried a `summary`, used by the Astro layout.
crabbucket ignored all six silently.

A design system now declares what it reads, as
[`Theme::Extra`](../content/#frontmatter-your-design-system-adds), and a page
missing a required field fails naming itself. `crabbucket-theme-plain` reads a
`summary`, which is exactly cairn's case.

One thing still does not hold: a *misspelled* key vanishes rather than
failing, because serde cannot combine `flatten` with `deny_unknown_fields`.

### Pages that are not Markdown — open

Four of cairn's ten pages are `.astro` files: the landing page, and three that
compose prose with layout in ways a Markdown file cannot express.

crabbucket has nowhere to put them. Every route comes from a file in
`content/`, so the workaround is to rewrite each as Markdown plus directives —
which loses exactly the thing that made them components.

This is the largest gap, and it is a gap against Astro specifically: Astro has
content collections *and* pages, and crabbucket has only the first.

### Directives cannot read site data — open

`<Terminal name="mcp" />` looks a recording up by name, from data the site
generates. A crabbucket directive sees its attributes and its body and nothing
else, so there is no way to write that one at all.

## The count

Ten pages. Six migrated mechanically. One bug found and fixed. Three gaps
found and filed; one is now fixed, and one of the remaining two blocks four of
the ten pages.

That is the honest number, and it is the number worth publishing: a migration
guide that claims everything is easy is a guide nobody trusts twice.

:::callout{kind = "note", title = "What was not done"}
cairn's repository was not modified. The migration was performed against a
read-only clone, because adopting a framework is the maintainer's decision and
this was an exercise in finding out what breaks.
:::
