---
id: 34
title: The book
type: docs
status: done
milestone: v1.0
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: l
area: docs
---

## Problem

The site documents the parts. Nobody has written the path through them — from
an empty directory to a deployed site with a house style, in order, with the
reasoning visible.

Reference documentation answers questions you already know to ask. It is the
wrong shape for the person who has not started yet.

## Proposal

A long-form guide, built with crabbucket, which is also the largest test the
framework will have had.

Roughly:

1. A site in five minutes
2. Content, frontmatter and collections
3. Routes, the base path, and why links are checked
4. Layouts, and why a layout is a type
5. Your own design system
6. A fleet of sites sharing one style
7. The dev loop
8. Deploying
9. Reference

Every code sample compiled by CI. A book with a snippet that no longer
compiles is worse than no book, because it is confidently wrong.

Write chapter 6 first. It is the one that will be worst, because it is the
one where the promise is largest.

## Acceptance criteria

- [x] Nine chapters, drafted and edited
- [x] Built with crabbucket, deployed alongside the site
- [x] Every code sample compiled in CI
- [x] Search works across the book
- [x] Chapter 6 written first and reviewed hardest

## 2026-09-08

Nine chapters plus a contents page, at /book/, following one project -- a docs site for a library called Ferrite -- from an empty directory to twelve repositories sharing one house style. Each chapter continues the last.

Chapter 6 was written first, as the item asked. It is the one where the promise is largest, and writing it first is what turned up the two things in it nobody writes down: that a design system's version numbers become something you hesitate over once twelve sites depend on them, and that twelve Cargo.lock files pinning twelve commits of one git dependency is not legibly twelve versions.

The "every code sample compiled in CI" criterion is met by inversion rather than by a checker. examples/book-samples is a real crate -- Ferrite, a third design system, plus the site-crate code chapter 6 shows -- that the workspace compiles and that has its own tests proving it renders a whole site. The book's Rust blocks ARE that crate's source, and tests/book.rs asserts each one appears in it character for character. So "does the book still compile?" is answered by cargo build, not by a snippet extractor that drifts. A fence marked rust sketch opts out for blocks that are deliberately not real code.

Ferrite is the third design system and the first that needed no changes to crabbucket to write, which is a fair signal the seam has settled.

Search covers all ten book pages (index 114KB, under the 300KB warning). Links across the book and into the docs are checked like everything else: 27 pages, 1196 links.

Also fixed while in there: docs/content.md still said directives were unimplemented. They have been since 0.3.
