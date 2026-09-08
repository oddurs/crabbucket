---
id: 44
title: Layouts as an associated type on Theme
type: feature
status: done
milestone: v0.0
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: m
area: theme
---

`Theme::Layout` is an associated type and `PageMeta<L>` is generic over it,
so a page's `layout` field deserializes into the theme's own enum.

A misspelled layout produces serde's `unknown variant \`dcos\`, expected one
of \`page\`, \`docs\`, \`landing\`` — with no registry, no list of valid names
to maintain, and no code in crabbucket that knows the word "docs".
