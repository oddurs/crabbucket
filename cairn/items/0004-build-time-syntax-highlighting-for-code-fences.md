---
id: 4
title: Build-time syntax highlighting for code fences
type: feature
status: done
milestone: v0.1
labels:
- docs-quality
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: content
---

## Problem

A documentation framework whose code blocks are grey is not finished. Every
page in these docs is mostly code.

The usual answer is a JavaScript highlighter, which contradicts the whole
zero-JS position: shipping 40KB of Prism to colour text that was already
static at build time is exactly the trade this project exists to refuse.

## Proposal

Highlight at build time. The language tag is already preserved on the fence,
so the information is there.

Candidates:

- **syntect** — mature, Sublime grammars, heavy build dependency, and a
  well-trodden path (mdBook, Zola). The default choice.
- **tree-sitter** — better parses, much more assembly required, one grammar
  crate per language.
- **two-face** — syntect grammar bundle with better licensing hygiene.

Emit `<span class="tok-…">` classes rather than inline styles, and define
those classes from `design/tokens.toml` like everything else. That keeps
highlighting themeable by the same mechanism as the rest of the design
system, and keeps colours out of the HTML.

Watch the build time: syntect's default `SyntaxSet` load is not free. Load it
once per build, not once per fence, and consider `onig`-free features.

## Acceptance criteria

- [x] Fenced blocks with a known language are highlighted
- [x] An unknown or absent language renders as plain, not as an error
- [x] Highlight colours come from tokens, not from a hard-coded theme
- [x] Zero client-side JavaScript is added
- [x] Building `examples/site` stays under a second

## 2026-09-08

Done with syntect, default-features off plus default-fancy, so no C dependency. ClassedHTMLGenerator with SpacedPrefixed prefix 'tok-' emits scope classes; colours live in design/tokens.toml under [syntax] and nowhere else. Only top-level scope atoms are styled (comment, keyword, storage, string, constant, entity, support, variable, punctuation, invalid) because syntect emits every atom as a class and styling second-level atoms would collide at equal specificity. SyntaxSet loads once per process via OnceLock.

## 2026-09-08

Measured: 0.09s to build the ten-page site in release, cold run 0.31s including the SyntaxSet load. Well inside the one-second budget.
