---
id: 41
title: Base-path-aware URLs applied in one place
type: feature
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: routing
---

`Url` is the only thing that concatenates a base path, and it implements
`maud::Render` so components interpolate it directly.

Every spelling of a base — `repo`, `/repo`, `repo/`, `/repo/` — normalises
identically, so the configuration file cannot express the bug either.

Verified live: the site is served from `/crabbucket/` and every asset and
route resolves.
