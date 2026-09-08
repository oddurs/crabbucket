---
id: 37
title: 'Islands: typed props across the hydration boundary'
type: feature
status: backlog
milestone: v2.0
depends_on:
- 70
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: xl
area: ui
---

## Problem

This was filed as an exclusion and closed with "reopen this when there is a
specific page that needs it. 'It would be nice' is not that."

Two things have changed, and neither is "it would be nice".

**The first is that the exclusion is now the only untyped seam in the
project.** A page's frontmatter is a struct, its layout is a variant, its
links are checked before the build is allowed to succeed. Then the search box
is hand-written JavaScript reading a JSON file whose shape is agreed between a
Rust function and a JavaScript function by hand, and if the index format
changes the failure arrives in a browser in front of a reader. That is the
exact class of failure this project exists to refuse, sitting inside the
project. The concrete page that needs this is the documentation site's own
search.

**The second is that the typed boundary turns out to be the differentiator,
not the cost.** Reading how the alternatives do it:

- **Astro** serialises island props through a hand-written tag table —
  `PROP_TYPE = { Value, JSON, RegExp, Date, Map, Set, BigInt, URL,
  Uint8Array, Uint16Array, Uint32Array, Infinity }` — with cyclic-reference
  detection that throws *during render*, and a mirror-image decoder in the
  client kept in sync by hand. The set of types that may cross the boundary is
  an enumeration of twelve.
- **Leptos** already proves the Rust version works. `#[island]` compiles only
  island code to wasm; props pass from a server component to an island as the
  *same Rust type*; `children` can be server-rendered and projected into an
  island without the island hydrating them, so server-only content nests
  inside islands inside server content. There is `#[island(lazy)]` for
  deferred loading.

So the thing to build is not "Astro islands in Rust". It is the boundary
Astro cannot have: **props are one type, `Serialize` on the server and
`Deserialize` in the browser, from one definition.** A mismatch is a compile
error. There is no table to fall off, and a cycle is not representable.

Nobody currently occupies this. Hugo and Zola have no client story at all.
Astro's is validated at runtime. Leptos and Dioxus have the typed boundary and
no content pipeline, no design-system seam, and no build gates.

## Proposal

An island is a component a design system marks as interactive.

```rust sketch
pub trait Island: Serialize + DeserializeOwned {
    const NAME: &'static str;
    fn render(&self) -> Markup;
    fn hydrate(self, root: &Element);
}
```

The server renders `render()` into the page, writes the props beside it, and
emits a mount point. The wasm client reads the props *as the same type* and
calls `hydrate`. The static render is the content, not a placeholder — a
reader with no JavaScript gets a working page, which is the existing
zero-JavaScript default holding rather than being abandoned.

Decisions to make deliberately, each of which is a place this could go wrong:

- **When to hydrate is declared at the call site, not in the component.**
  Astro's `client:load` / `client:idle` / `client:visible` / `client:media`
  vocabulary is the right idea — the component does not know how urgent it is;
  the page does.
- **No reactive runtime, initially.** Leptos's wasm cost is largely its
  reactive system. An island that mounts, reads typed props and attaches
  handlers needs none of it. Reach for one only when a real island needs it.
- **The props are serialised once and shared.** Two islands on a page with the
  same props should not embed them twice.
- **Islands are opt-in per design system**, behind a trait default that says
  "not this one", exactly as the router and search clients already are. A
  design system that ships no islands must compile no wasm.

## Acceptance criteria

- [ ] An `Island` trait; props cross as one Rust type, both directions derived
- [ ] A props type that does not round-trip is a compile error, with a test
      that asserts the failure
- [ ] The static render is complete without JavaScript, asserted for every
      island in the tree
- [ ] Hydration timing declared at the call site
- [ ] A design system with no islands compiles no wasm and ships none
- [ ] `doc/STABILITY` covers, or explicitly declines to cover, the new surface

## 2026-09-08

Reopened after reading how the alternatives do it. Astro serialises island props through a hand-written twelve-entry PROP_TYPE tag table with runtime cycle detection that throws during render; Leptos already passes props server-to-island as the same Rust type, with server children projected into islands without hydration, and has island(lazy). So the typed boundary is achievable and is the differentiator rather than the cost. The concrete page that needs it is this site's own search box, which is currently hand-written JavaScript reading a JSON file nothing validates -- the one untyped seam left in the project.
