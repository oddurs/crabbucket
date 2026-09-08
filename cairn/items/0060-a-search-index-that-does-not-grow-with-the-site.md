---
id: 60
title: A search index that does not grow with the site
type: feature
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: ui
---

## Problem

`search.json` is a JSON array holding the full text of every page. Every
visitor who might search downloads all of it before they can type. On this
site it is 114KB for 27 pages; `search::SIZE_WARNING` fires at 300KB, which
is a warning that the design has run out rather than a limit worth having.

The cost is linear in the size of the site, paid by every reader, to answer a
query that touches two or three pages.

## Proposal

Adopt the shape Pagefind uses, which is the right one and is already proven in
Rust. From `pagefind/src/index/`:

- **An inverted index**, not a document list. `PackedWord { word, pages:
  Vec<PackedPage>, additional_variants }`, where `PackedPage { page_number,
  locs, meta_locs }` — a word maps to the pages and positions it occurs at.
- **Chunked by alphabetical word range.** A small meta index holds
  `MetaChunk { from, to, hash }`; the client loads the meta index, finds the
  one chunk whose range covers the word being searched, and fetches only
  that. Default chunk size is 20,000 words.
- **Page content lives in separate fragments**, fetched only for the results
  actually shown.
- **Binary, not JSON.** Pagefind uses `minicbor`. Page numbers are
  delta-encoded within each word's page list before encoding.
- **Content-addressed filenames**, so every chunk is immutably cacheable and
  a rebuild only invalidates what changed.

For a site of this size the first load becomes a meta index of a few
kilobytes plus one chunk, instead of the whole corpus.

Two deliberate differences from Pagefind. It indexes *rendered HTML* as a
post-process, because it has to work with any generator; crabbucket has the
typed page in hand at render time, including its headings, so it can index
from the structured value and skip the parse. And Pagefind's filters and
sorts are configured with `data-pagefind-*` attributes in markup; here they
should come from frontmatter, where they are typed.

The JavaScript client is rewritten against the new format in this item, and
becomes an island in [[0058]] rather than staying hand-written forever.
`SIZE_WARNING` can then be deleted, because the size stops being the reader's
problem.

## Acceptance criteria

- [ ] An inverted index, chunked by word range, with a meta index
- [ ] Fragments separate from the index, fetched per displayed result
- [ ] Binary encoding, content-addressed filenames
- [ ] The client fetches the meta index plus only the chunks a query needs
- [ ] Measured: first-load bytes for a query, on 27 pages and on 521
- [ ] Filters and sorts declared in frontmatter, typed
- [ ] `search::SIZE_WARNING` removed, and the reason recorded in NEWS
