---
id: 11
title: '`crab build --out` and `--base` overrides'
type: feature
status: done
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

- [x] `--out` writes elsewhere and leaves `dist/` alone
- [x] `--base` overrides config and normalises identically
- [x] `--` ends option processing
- [x] An unknown option exits 2 with `Try 'crab --help'`
- [x] `--help` documents both

## 2026-09-08

Done. build_with(dir, theme, Options) is the general form; build() is the two-argument convenience. --base goes through Config::set_base so a command-line override and a config file normalise identically. Argument parsing stayed hand-rolled: it is one screen, it handles -- correctly, and nine tests cover it.
