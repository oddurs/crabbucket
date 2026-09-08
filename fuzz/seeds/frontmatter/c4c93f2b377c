+++
title = "Feeds"
layout = "docs"
order = 10
+++

# Feeds

```toml
# site.toml
url = "https://you.github.io/project/"

[[feed]]
collection = "notes"
title = "Notes"
limit = 20
```

That writes `notes/feed.xml` and `notes/atom.xml`, and puts
`<link rel="alternate">` tags in every page's head.

Both formats, because readers disagree about which they want and both are
small. RSS 2.0 gets RFC 822 dates and Atom gets RFC 3339 ones, which is the
only real difference between writing one and writing two.

## A feed is how a site says a collection is dated

Most pages are not dated, so `date` is optional in frontmatter. Configuring a
feed makes it required for the pages that feed carries:

```
crab: content/notes/undated.md: a feed is configured for `notes`, so this
page needs a `date`
```

That is the same bargain as everything else here — say what you want and the
build holds you to it.

## Dates are dates

TOML has them, so frontmatter carries a real one:

```markdown
+++
title = "Why routes became a type"
date = 2026-09-08
+++
```

A malformed date fails at the file and the line without the feed code doing
any checking, because `serde` already did it.

A date with no time is **midnight UTC**. Reading the build machine's time zone
would make the same content produce different feeds on different machines.

:::callout{kind = "note", title = "The feed's own timestamp"}
Atom's channel-level `<updated>` is the newest entry's date, not the time of
the build. Otherwise every build would differ from the last one for no reason,
and every reader would think something had changed.
:::

## No absolute URL, no feed

A feed is read somewhere else, so its links have to be absolute. Without a
`url` in `site.toml` nothing is written, the head gets no tags, and the build
says so:

```
crab: warning: site.toml configures a feed but has no url; a feed needs
absolute URLs, so none was written
```

The dates are not demanded either, since no feed is being built.

## What a collection is

The `collection` is a route prefix. `notes` covers `notes/one` and
`notes/deep/two`, and deliberately *not* `notes` itself — the index page is
the thing the feed is for, not an entry in it.

An empty `collection` covers the whole site.
