# Makefile for crabbucket -- a thin GNU-style front end over Cargo.
# Copyright (C) 2026 Oddur Sigurdsson
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The usual GNU variables are honoured:
#
#   make install prefix=/usr DESTDIR=/tmp/stage

PACKAGE = crabbucket
PROGRAM = crab
SITE    = examples/site

CARGO = cargo
INSTALL = install
INSTALL_PROGRAM = $(INSTALL)
INSTALL_DATA = $(INSTALL) -m 644

prefix = /usr/local
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
datarootdir = $(prefix)/share
docdir = $(datarootdir)/doc/$(PACKAGE)
mandir = $(datarootdir)/man
man1dir = $(mandir)/man1

DESTDIR =

CARGO_PROFILE = release
CARGO_FLAGS = --release
TARGETDIR = target/$(CARGO_PROFILE)

DOCS = README NEWS AUTHORS THANKS ChangeLog COPYING doc/DESIGN

.PHONY: all check fmt lint test site roadmap install uninstall clean distclean dist help

all:
	$(CARGO) build --workspace $(CARGO_FLAGS)

check: fmt lint test site roadmap

fmt:
	$(CARGO) fmt --all --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

test:
	$(CARGO) test --workspace --all-targets

site:
	$(CARGO) run -q -p crabbucket-cli -- build $(SITE)

# Validate the roadmap items against cairn.toml, if cairn is installed.
roadmap:
	@command -v cairn >/dev/null && cairn check || echo 'cairn not installed; skipping'

install: all
	$(INSTALL) -d $(DESTDIR)$(bindir)
	$(INSTALL_PROGRAM) $(TARGETDIR)/$(PROGRAM) $(DESTDIR)$(bindir)/$(PROGRAM)
	$(INSTALL) -d $(DESTDIR)$(docdir)
	$(INSTALL_DATA) $(DOCS) $(DESTDIR)$(docdir)
	$(INSTALL) -d $(DESTDIR)$(man1dir)
	$(INSTALL_DATA) doc/$(PROGRAM).1 $(DESTDIR)$(man1dir)/$(PROGRAM).1

uninstall:
	rm -f $(DESTDIR)$(bindir)/$(PROGRAM)
	rm -f $(DESTDIR)$(man1dir)/$(PROGRAM).1
	rm -rf $(DESTDIR)$(docdir)

clean:
	$(CARGO) clean
	rm -rf $(SITE)/dist

distclean: clean
	rm -f Cargo.lock

dist:
	$(CARGO) package --workspace

help:
	@echo 'Targets: all check fmt lint test site roadmap install uninstall clean distclean dist'
