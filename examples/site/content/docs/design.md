+++
title = "Design"
layout = "docs"
order = 10
+++

# Design

This page is the short version. The long one is
[`doc/DESIGN`](https://github.com/oddurs/crabbucket/blob/main/doc/DESIGN) in
the repository, which is the document this framework is being built against.

## The thesis

**A site is a typed value.**

Rust's advantage here is not speed. Any generator is fast enough for a
forty-page docs site; Hugo builds it in 30ms and nobody is unhappy. The
advantage is that Rust can make the whole site a typed value, so an entire
class of bug stops being a runtime surprise:

- frontmatter is a struct, not a map
- a layout is an enum variant, not a name
- a URL is a `Url`, not a `String`
- a colour is a token constant, not a hex literal
- a link is checked before the build is allowed to succeed

## What is borrowed, and from where

| From | What |
|---|---|
| Astro | content collections, zero JS by default, layouts. The model, not the file format. |
| Next.js | file-based routing, client navigation with no full reload |
| Zola, Hugo | one binary, `+++` TOML frontmatter, builds that feel instant |
| SvelteKit | prerender by default, a real answer for base paths |
| Style Dictionary | design tokens as one source, compiled out to every consumer |
| maud | HTML as a typed expression, not a string |
| htmx | the small "just swap the body" router, and nothing more |
| turborust | the dev loop — [do not rebuild it](../dev-loop/) |

## Why not `.astro` files

Astro's authoring experience is the best in the category, and the first
instinct is to reproduce the file format. That is the wrong half to copy.

The appealing part of Astro is the *model*: typed content collections, zero
JavaScript unless asked, layouts as composition. The expensive part is the
*compiler*. A single-file component format needs a parser, scoped-style
extraction, and — the killer — an evaluator for expressions inside the
template. In Rust that means shipping an interpreter or generating code from a
bespoke syntax, and both are strictly worse than writing Rust.

So: Astro's ergonomics for content, Rust for components, and directives as the
bridge between them.

## Fast by construction, no-refresh by choice

The output is plain static HTML and CSS. Zero JavaScript, works in a text
browser.

Opt in, per site, to a router of about a kilobyte and a half, gzipped: intercept same-origin
clicks, fetch, swap `<main>`, push history, drive the View Transitions API for
the cross-fade. That is the Next.js feel with no hydration, no virtual DOM and
no framework. With JavaScript off it degrades to ordinary navigation, because
it *is* ordinary navigation with a click handler in front of it.

Wasm islands are a later escape hatch, deliberately out of v0. A landing page
and a docs site do not need them, and adding them early would warp every other
decision.

## What is decided but not built

- **Generated OpenGraph images**, so a shared link is not a grey rectangle.

Everything else this section has listed since the first draft is now built:
[typed routes](../routing/), [directives](../components/),
[search](../search/), a table of contents, and a light palette. This list is
kept honest by being short.

The tracked, ordered version of it is
[ROADMAP.md](https://github.com/oddurs/crabbucket/blob/main/ROADMAP.md).

## What has changed since the first draft

Five sections of `doc/DESIGN` have been overruled by building the thing.

Section 3 originally described only the `Route` enum; content has no compiler,
so link checking became a build gate in its own right rather than a lesser
version of the same idea. Both halves now exist, and neither replaces the
other.

Section 7 claimed the router was "about a kilobyte" for three revisions
without anybody measuring it. It is three.

Section 6 promised that unused component CSS would never be emitted. That
promise was withdrawn: at this size it optimises the wrong number, and the
problem worth solving was collision, not size.

Section 10 called for a `css!` macro. There is no macro — a struct and a
string replacement did the whole job, and a proc-macro dependency would have
bought nothing.

Two of those three revisions removed work rather than adding it. A design note
that has never been overruled by contact with the code is a design note nobody
was reading.

## What is deliberately excluded

wasm islands, i18n, image optimisation, a plugin system, incremental builds,
and arbitrary expressions in Markdown. Each is a good feature and each would
cost more than it returns for a landing page with some docs next to it.
