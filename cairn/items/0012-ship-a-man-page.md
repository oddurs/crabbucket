---
id: 12
title: Ship a man page
type: docs
status: backlog
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: s
area: docs
---

## Problem

The project follows the GNU conventions everywhere else — `COPYING`,
`ChangeLog`, `NEWS`, `--help` and `--version` in the documented shape, a
Makefile with `prefix` and `DESTDIR`. A GNU-shaped program without a man page
is conspicuous, and `make install` currently installs a binary that `man`
knows nothing about.

## Proposal

`doc/crab.1`, written by hand in roff rather than generated. It is one
command with a handful of options; a generator would be more machinery than
the thing it generates.

Sections: NAME, SYNOPSIS, DESCRIPTION, OPTIONS, EXIT STATUS, FILES,
EXAMPLES, REPORTING BUGS, SEE ALSO (turborust, cargo).

Document the exit statuses explicitly, since they are load-bearing: 0
success, 1 build failure, 2 usage error.

Install to `$(mandir)/man1` from the Makefile, and add `mandir` alongside the
`prefix` variables already there.

## Acceptance criteria

- [ ] `doc/crab.1` exists and renders without warnings under `man --warnings`
- [ ] `make install` installs it to `$(mandir)/man1`
- [ ] `make uninstall` removes it
- [ ] Exit statuses documented
- [ ] Options match `crab --help` exactly
