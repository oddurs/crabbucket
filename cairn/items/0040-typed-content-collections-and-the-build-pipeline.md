---
id: 40
title: Typed content collections and the build pipeline
type: feature
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: build
---

Markdown with `+++` TOML frontmatter, deserialized into a type rather than a
map. Routes derived from paths, with any `index.md` taking its directory's
route. Every route written as a directory holding an `index.html`.

`Error` carries a path in every variant, so a build error that does not name
its file cannot be constructed.
