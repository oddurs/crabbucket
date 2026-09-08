---
id: 12
title: Ship a man page
type: docs
status: done
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

- [x] `doc/crab.1` exists and renders without warnings under `man --warnings`
- [x] `make install` installs it to `$(mandir)/man1`
- [x] `make uninstall` removes it
- [x] Exit statuses documented
- [x] Options match `crab --help` exactly

## 2026-09-08

Done. doc/crab.1, written by hand in roff -- a generator would be more machinery than the thing it generates. Lints clean under mandoc -T lint. make install puts it in $(mandir)/man1 and make uninstall removes it.

The last acceptance criterion, that the options match crab --help exactly, is now a test rather than a promise: it scans both texts for long options and compares the sets. Verified it fails when they disagree by renaming --force to --forcey in the roff and watching it complain.
