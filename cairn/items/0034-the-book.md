---
id: 34
title: The book
type: docs
status: backlog
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

- [ ] Nine chapters, drafted and edited
- [ ] Built with crabbucket, deployed alongside the site
- [ ] Every code sample compiled in CI
- [ ] Search works across the book
- [ ] Chapter 6 written first and reviewed hardest
