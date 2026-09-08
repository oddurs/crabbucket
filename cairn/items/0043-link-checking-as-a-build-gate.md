---
id: 43
title: Link checking as a build gate
type: feature
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: routing
---

Every internal `href` and `src` is resolved after rendering and looked up
against the route set or the asset set. A dead link fails the build, with
every offender listed at once and each naming the page that wrote it.

Not a warning, and no flag to disable it. External links are not checked.

It found two bugs in its own documentation on the first run, which is the
best possible argument for it.
