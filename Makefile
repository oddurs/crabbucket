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

DOCS = README NEWS AUTHORS THANKS ChangeLog COPYING doc/DESIGN doc/STABILITY

.PHONY: all check fmt lint test doc-test cover audit api fuzz mutants \
        site roadmap install uninstall clean distclean dist help

# `cargo test' is the fallback: nextest is better in every way that matters
# here -- a process per test, a timeout, and a summary you can read -- but it
# is a separate installation, and a contributor who has not installed it
# should still be able to run the suite.
NEXTEST := $(shell command -v cargo-nextest 2>/dev/null)
COVER_FLOOR = 94

# The crates that go to crates.io, in dependency order.  `bench' and the two
# examples are `publish = false'.  crates/crabbucket/tests/packaging.rs asserts
# this list matches the manifests, so it cannot drift from them in silence.
PUBLISHED = crabbucket-routes crabbucket-tokens crabbucket-og \
            crabbucket crabbucket-ui crabbucket-theme-plain crabbucket-cli

all:
	$(CARGO) build --workspace $(CARGO_FLAGS)

check: fmt lint test site roadmap

fmt:
	$(CARGO) fmt --all --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

# Two commands, because nothing runs both.  `--all-targets' excludes doctests
# -- quietly, and it is the default thing to write -- so every `///' example in
# the public API went unverified until this line existed.
test: doc-test
ifdef NEXTEST
	$(CARGO) nextest run --workspace --all-targets
else
	@echo 'cargo-nextest not installed; falling back to cargo test'
	$(CARGO) test --workspace --all-targets
endif

doc-test:
	$(CARGO) test --workspace --doc

# A number, so that it falling is something anybody can notice.  bench/ is a
# measuring tool rather than the product and is not counted.
cover:
	$(CARGO) llvm-cov --workspace --all-targets --ignore-filename-regex '(^|/)bench/' \
	  --fail-under-lines $(COVER_FLOOR) --summary-only

cover-html:
	$(CARGO) llvm-cov --workspace --all-targets --ignore-filename-regex '(^|/)bench/' \
	  --open

# Licences, advisories, duplicate dependencies and where crates came from.
# This is a GPL-3.0-or-later project: a GPL-incompatible transitive licence is
# a licensing bug, not a preference.
audit:
	$(CARGO) deny --all-features check

# The four parsers that run on whatever is in a content file, on arbitrary
# bytes.  Nightly and a sanitizer, so `fuzz/' is not a workspace member.
#
#   make fuzz                 all four, briefly
#   make fuzz FUZZ_TIME=600   one long run
#   make fuzz FUZZ=markdown   just the one
FUZZ_TIME = 60
FUZZ = markdown frontmatter directives links

fuzz:
	@for target in $(FUZZ); do \
	  echo "==> $$target"; \
	  mkdir -p fuzz/corpus/$$target; \
	  ( cd fuzz && cargo +nightly fuzz run $$target corpus/$$target seeds/$$target \
	      -- -max_total_time=$(FUZZ_TIME) ) || exit 1; \
	done

# Whether the tests would notice.  Slow -- it rebuilds once per mutation --
# so it is a scheduled job rather than part of `make check'.
mutants:
	$(CARGO) mutants --workspace --no-shuffle -j 4

# What `doc/STABILITY' promises, checked against the branch this one came from
# rather than against a release, because there is not one yet.
api:
	$(CARGO) semver-checks $(addprefix -p ,$(PUBLISHED)) \
	  --baseline-rev $$(git merge-base HEAD origin/main 2>/dev/null || echo HEAD~1)

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
	@echo 'Build:    all install uninstall clean distclean dist'
	@echo 'Check:    check fmt lint test doc-test site roadmap'
	@echo 'Measure:  cover cover-html audit api fuzz mutants'
