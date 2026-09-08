---
id: 62
title: 'An asset pipeline: images that are enqueued, hashed and typed'
type: feature
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: build
---

## Problem

`static/` is copied verbatim and that is the whole asset story. A site with a
2400px screenshot ships a 2400px screenshot to a phone. There is no resizing,
no modern format, no cache-busting, and no way for a component to know an
image's dimensions — so every `<img>` is a layout shift waiting to happen.

## Proposal

The architecture for this already exists in the tree, for social cards: the
render pass pushes `(path, bytes)` onto `cards`, and a later pass writes them.
Zola's image pipeline is the same pattern done properly, and worth copying
closely:

- `ResizeOperation` and `ResizeInstructions` derive `Hash`, and the output
  filename is derived from the hash of `(input path, source, operation,
  format, filter)`. The result is content-addressed and immutably cacheable.
- Templates call `enqueue(...)` during rendering and get an `EnqueueResponse
  { url, static_path, width, height, orig_width, orig_height }` back
  immediately. The work happens later, in a batch, in parallel — `do_process`.
- `prune()` deletes outputs that no longer correspond to a live operation, so
  the cache does not grow forever.

The part worth taking further than Zola: because the enqueue returns the
dimensions, a typed API can *require* them at the call site. A component that
renders an image cannot omit `width` and `height`, because the function that
gives it a URL also gives it the numbers, and the type says so. Layout shift
stops being a lint and becomes unrepresentable — which is the same move as
`Url` and the base path.

Scope for this item:

- An `Asset` seam: enqueue during render, process after, content-addressed
  output, `prune` for stale entries, cached in `.crabbucket/` beside the cards
- Images: resize, and re-encode to AVIF or WebP with the original as fallback
- A typed component API that hands back a URL *and* dimensions together
- CSS and JS: hash the filename, so `site.css` can be cached forever. Minify
  behind a flag, not by default — the stylesheet is generated from `Style`
  values and is already small.

Not in scope: bundling, a module graph, or anything resembling Vite. There is
one stylesheet and at most two scripts.

## Acceptance criteria

- [ ] Assets enqueued during render and processed in a later pass
- [ ] Output filenames content-addressed; `prune` removes stale ones
- [ ] Images resized and re-encoded, original kept as fallback
- [ ] A component cannot render an image without its dimensions
- [ ] Cached across builds; the timing cost measured and published
- [ ] Deterministic: the same input produces the same bytes, and the existing
      two-directory determinism test covers assets too
