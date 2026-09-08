---
id: 11
title: '`crab build --out` and `--base` overrides'
type: feature
status: backlog
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: cli
---

## Problem

`base` is baked into `site.toml`, so a preview deploy under a different path
means editing a tracked file. And output always goes to `dist/`, which makes
building two variants awkward.

## Proposal

    crab build [DIR] [--out PATH] [--base PATH]

Both override the config for one invocation and nothing else. `--base`
normalises through the same function the config does, so every spelling
behaves identically.

Keep the hand-rolled argument parsing. It is forty lines, it follows the GNU
conventions already documented in `HACKING`, and a dependency that pulls in
a derive macro to save those forty lines is a bad trade for a tool this size.

## Acceptance criteria

- [ ] `--out` writes elsewhere and leaves `dist/` alone
- [ ] `--base` overrides config and normalises identically
- [ ] `--` ends option processing
- [ ] An unknown option exits 2 with `Try 'crab --help'`
- [ ] `--help` documents both
