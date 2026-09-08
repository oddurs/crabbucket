---
id: 42
title: Design tokens generated into Rust and CSS from one file
type: feature
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: ui
---

`design/tokens.toml` is read by `crabbucket-ui`'s build script, which emits
Rust constants and CSS custom properties from the same pass. They cannot
drift, and a renamed token fails to compile at every use site.

Each constant is a `var(--name, fallback)` reference, so runtime theming
still works.
