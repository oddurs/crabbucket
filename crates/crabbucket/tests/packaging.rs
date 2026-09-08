// Copyright (C) 2026 Oddur Sigurdsson
//
// This file is part of crabbucket.
//
// crabbucket is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// crabbucket is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program.  If not, see <https://www.gnu.org/licenses/>.

//! What the published crates promise about themselves.
//!
//! Everything here is checkable without a network and without publishing,
//! which is the point: the alternative is finding out from crates.io, and a
//! version can be yanked but never taken back.

use std::fs;
use std::path::{Path, PathBuf};

/// The crates that are published.  `bench' and `examples/site-crate' are not.
const PUBLISHED: [&str; 7] = [
    "crabbucket",
    "crabbucket-cli",
    "crabbucket-og",
    "crabbucket-routes",
    "crabbucket-theme-plain",
    "crabbucket-tokens",
    "crabbucket-ui",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn manifest(crate_name: &str) -> String {
    let path = root().join("crates").join(crate_name).join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn every_published_crate_ships_the_licence_it_claims() {
    // GPLv3 section 4 asks that every recipient of the source get the licence
    // with it, and a .crate file is source.  `include' cannot reach outside
    // the package, so each crate carries a copy -- and a copy that is allowed
    // to drift is worse than no copy at all.
    let canonical = fs::read_to_string(root().join("COPYING")).expect("no COPYING");

    for name in PUBLISHED {
        let path = root().join("crates").join(name).join("COPYING");
        let copy = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}; `cp COPYING crates/{name}/'", path.display()));

        assert_eq!(copy, canonical, "crates/{name}/COPYING has drifted");
        assert!(
            manifest(name).contains("\"COPYING\","),
            "crates/{name} does not `include' its COPYING, so it would not ship"
        );
    }
}

#[test]
fn every_published_crate_says_what_ships() {
    // An allow-list, so a new directory has to be added on purpose rather
    // than being published because nobody noticed it.
    for name in PUBLISHED {
        assert!(
            manifest(name).contains("\ninclude = ["),
            "crates/{name} has no `include', so it publishes whatever is lying around"
        );
    }
}

#[test]
fn no_published_crate_ships_its_tests() {
    // Several test files read repository files -- doc/STABILITY, doc/crab.1,
    // this very file -- which a package cannot reach.  Shipping them would
    // publish a test suite that cannot compile.
    for name in PUBLISHED {
        assert!(
            !manifest(name).contains("\"tests/"),
            "crates/{name} ships tests/, which reads files a package does not have"
        );
    }
}

#[test]
fn every_published_crate_has_the_metadata_crates_io_shows() {
    for name in PUBLISHED {
        let manifest = manifest(name);

        for field in [
            "description",
            "keywords",
            "categories",
            "repository",
            "readme",
        ] {
            assert!(
                manifest.contains(&format!("\n{field}")),
                "crates/{name} has no `{field}'"
            );
        }
    }
}

#[test]
fn the_readme_is_markdown_because_that_is_what_crates_io_renders() {
    // crates.io renders a readme with no extension *as Markdown*, not as
    // plain text.  This one is written GNU-style -- setext headings and
    // indented blocks -- which is valid Markdown only while every indented
    // block is indented by four spaces.  At two, CommonMark reflows them into
    // the paragraph above and the alignment that carries the meaning is gone.
    let readme = fs::read_to_string(root().join("README")).expect("no README");

    for (number, line) in readme.lines().enumerate() {
        let indent = line.len() - line.trim_start().len();

        assert!(
            indent == 0 || indent >= 4,
            "README:{}: indented by {indent}, which CommonMark reflows: {line:?}",
            number + 1
        );
    }

    // A lone backtick pairs with the next one and swallows everything
    // between, which is what `like this' used to do here.
    for (number, line) in readme.lines().enumerate() {
        if line.starts_with("    ") {
            continue;
        }

        assert_eq!(
            line.matches('`').count() % 2,
            0,
            "README:{}: an odd number of backticks: {line:?}",
            number + 1
        );
    }
}
