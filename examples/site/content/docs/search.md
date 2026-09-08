+++
title = "Search"
layout = "docs"
order = 9
+++

# Search

```toml
# site.toml
search = true
```

That writes two files — `search.json` and `search.js` — and puts a search box
in the masthead. Opt-in, like the [router](../design/), and for the same
reason: search needs script, and a page that ships none is the thing worth
defaulting to.

## The index is just the pages

For a corpus of tens of pages, an inverted index is the wrong shape. A linear
scan over a few tens of kilobytes of JSON is genuinely faster than parsing an
index would be, and it is a tenth of the code.

So the index is the pages: a URL, a title, the headings with their ids, and
the body with the markup taken out.

```json
[{"u":"/crabbucket/docs/routing/","t":"Routing","h":[["Routes come from paths","routes-come-from-paths"]],"b":"Routing …"}]
```

This site's index is 33KB, or 12KB over the wire.

:::callout{kind = "note", title = "The trade has a limit"}
Past 300KB the build says so rather than quietly shipping a slow page. The
answer at that size is a real index, not a bigger download — and that is the
reader's problem to be told about, not the build's to decide silently.
:::

## Two things the extraction gets right

Both were bugs first.

**Inline markup does not break a word.** Syntax highlighting wraps every token
in a `<span>`, so treating every tag as a word boundary turns
`serde::Deserialize` into three words and makes it unfindable. Block tags do
separate words; inline ones do not.

**Heading permalinks are dropped.** The `#` beside every heading is not a word
anyone searches for, and it was landing in the index between every heading and
the paragraph after it.

## The client

Fetched on first interaction rather than on page load, so a reader who never
searches never pays for it.

Ranking is title first, then headings, then body — a page whose *title*
matches is almost always the one you meant. A heading match links straight to
that heading rather than to the top of the page.

Keyboard throughout: `/` focuses the box from anywhere, arrows move through
results, enter follows, escape clears and closes.

## Without JavaScript

The form is emitted with `hidden` and the client removes it. A reader without
script sees no search box, rather than one that does nothing.
