---
id: 46
title: GitHub Pages deployment
type: chore
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: ci
---

A Pages workflow that builds with `--release` and uploads `dist/`, plus a CI
workflow running fmt, clippy, the test suite and a build of the site with
warnings denied.

`.nojekyll` is written by the build, so Pages does not run the output through
Jekyll and silently drop anything beginning with an underscore.
