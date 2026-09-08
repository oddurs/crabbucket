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

//! What Plain renders, as a diff.
//!
//! `seam.rs' asserts that this design system can build a site written for the
//! other one, which is the claim the crate exists to make.  It does that with
//! `contains()', which answers "is the thing I thought of still there?" and
//! never notices anything else moving.
//!
//! This answers the other question.  The fixture is small and stable on
//! purpose -- snapshotting the real documentation site would churn every time
//! somebody edited a paragraph, and a snapshot that churns is a snapshot
//! nobody reads before accepting.
//!
//!     cargo insta review

use std::fs;
use std::path::{Path, PathBuf};

use crabbucket::Options;
use crabbucket_theme_plain::Plain;

fn site(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    let write = |path: &str, body: &str| {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().expect("no parent")).expect("mkdir");
        fs::write(path, body).expect("write");
    };

    write(
        "site.toml",
        "title = \"Fixture\"\ndescription = \"A document.\"\nbase = \"/fixture/\"\n",
    );
    write(
        "content/index.md",
        "+++\ntitle = \"Home\"\nnav_order = 1\nsummary = \"What this page is for.\"\n+++\n\n\
         # Home\n\nProse, and a [chapter](/one/).\n",
    );
    write(
        "content/one.md",
        "+++\ntitle = \"One\"\nlayout = \"docs\"\nnav_order = 2\n+++\n\n\
         # One\n\n\
         :::callout{kind = \"warn\", title = \"Careful\"}\n\
         An indented note rather than a coloured box.\n\
         :::\n\n\
         ```rust\nfn main() {}\n```\n\n\
         Back [home](~/).\n",
    );

    dir
}

fn build(name: &str) -> PathBuf {
    let dir = site(name);
    let out = dir.join("dist");
    crabbucket::build_with(&dir, &Plain, Options::new().out_dir(out.clone()))
        .unwrap_or_else(|err| panic!("Plain should build the fixture, but: {err}"));

    out
}

/// One tag per line, so a snapshot diff points at what moved.
///
/// `maud' emits no whitespace between elements, which is right for a page and
/// useless for a diff: a fifteen-kilobyte single line reports every change as
/// "line 1 differs".  This is not pretty-printing -- no indentation, nothing
/// reordered -- only a line break where a reader would want one.
fn readable(html: &str) -> String {
    html.replace('<', "\n<").trim_start().to_string()
}

#[test]
fn a_rendered_page_is_what_it_was() {
    let html = fs::read_to_string(build("snapshot-page").join("one/index.html")).expect("no page");

    // The `docs' alias, this design system's callout, a highlighted fence and
    // a resolved `~/', in a document that ships no JavaScript at all.
    insta::assert_snapshot!("plain_page", readable(&html));
}

#[test]
fn the_stylesheet_is_what_it_was() {
    let css = fs::read_to_string(build("snapshot-css").join("site.css")).expect("no stylesheet");

    insta::assert_snapshot!("plain_stylesheet", css);
}
