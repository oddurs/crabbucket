---
id: 58
key: v2.0
title: The typed boundary reaches the browser
type: milestone
status: backlog
created: 2026-09-08
updated: 2026-09-08
priority: p1
---

The concept, finished.

"A site is a typed value" is true today of everything except the parts that
run in a browser. A page's frontmatter is a struct, its layout is a variant,
its links are checked — and then a search box is hand-written JavaScript that
the design system cannot type, reading a JSON file nothing validates.

This is the milestone where the boundary closes: an interactive component is
a Rust component, and the props that cross to it are one Rust type,
serialised on the server and deserialised in the browser from the same
definition.

Nobody else has this. Astro's props cross a hand-rolled twelve-entry tag
table with runtime cycle detection; Leptos has the typed boundary and no
content pipeline. The combination is the thing worth building.
