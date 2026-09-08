+++
title = "The book"
description = "From an empty directory to a fleet of sites sharing one house style, in order, with the reasoning visible."
layout = "docs"
order = 0
nav_order = 3
nav_label = "Book"
+++

# The book

The [documentation](~/docs/) answers questions you already know to ask. This
is the other shape: one path through the whole thing, in order, with the
reasoning left in.

It follows a single project — a docs site for a library called Ferrite —
from an empty directory to twelve repositories sharing one house style. Each
chapter continues the last. You can read it in an afternoon.

## The chapters

1. [A site in five minutes](a-site-in-five-minutes/) — `crab new`, and what
   the four files it writes are for
2. [Content](content/) — Markdown, frontmatter as a struct, collections and
   drafts
3. [Routes and links](routes-and-links/) — where a file ends up, what `~/`
   means, and why a dead link stops the build
4. [Layouts](layouts/) — why a layout is a type and not a string
5. [Your own design system](your-own-design-system/) — a whole one, from an
   empty crate
6. [A fleet of sites](a-fleet-of-sites/) — twelve repositories, one style,
   and what a version bump breaks
7. [The dev loop](the-dev-loop/) — why there is no `crab dev`
8. [Deploying](deploying/) — GitHub Pages, and the one thing that goes wrong
9. [Reference](reference/) — everything, on one page, to keep beside you

## About the code in it

Every Rust block in chapters 4, 5 and 6 is copied out of a real crate in
crabbucket's own repository — `examples/book-samples`, a design system called
Ferrite that the workspace compiles and that has its own test suite proving
it renders a site.

The copying goes one way and a test enforces it: if a block here is not in
that crate, character for character, CI fails. So a snippet cannot rot,
because a snippet is not a listing — it is the source.

A book with a snippet that no longer compiles is worse than no book, because
it is confidently wrong.

## Where to start

If you have never used crabbucket, start at chapter 1 and read forwards.

If you are here because you want the house-style-across-many-repositories
thing, read [chapter 6](a-fleet-of-sites/) first. It is the chapter this
whole framework exists for, and it will tell you quickly whether the rest is
worth your afternoon.
