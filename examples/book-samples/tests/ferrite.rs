// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ferrite builds a site.
//!
//! Chapter 5 of the book claims that the crate it walks you through is enough
//! to render a whole site.  Compiling proves the types line up; this proves
//! the claim.

use std::fs;
use std::path::{Path, PathBuf};

use crabbucket::{Options, Report};
use crabbucket_book_samples::Ferrite;

/// A fixture site, in a directory of its own.
///
/// Of its own because these tests run in parallel: one directory shared
/// between them is one test deleting the site another is building, which is
/// a failure that shows up only on whichever machine is busiest.
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
        "title = \"Ferrite\"\ndescription = \"A house style.\"\nbase = \"/fleet/\"\n",
    );
    write(
        "content/index.md",
        "+++\ntitle = \"Home\"\nnav_order = 1\nsummary = \"The front page.\"\n+++\n\n\
         # Home\n\nA [guide](/guide/) and a [chapter](/guide/first/).\n",
    );
    write(
        "content/guide/index.md",
        "+++\ntitle = \"Guide\"\nlayout = \"docs\"\norder = 1\nnav_order = 2\n+++\n\n# Guide\n",
    );
    write(
        "content/guide/first.md",
        "+++\ntitle = \"First\"\nlayout = \"docs\"\norder = 2\n+++\n\n\
         # First\n\n:::callout{title = \"Careful\"}\nMind the gap.\n:::\n\n\
         Back [home](~/).\n",
    );

    dir
}

fn build(name: &str) -> (Report, PathBuf) {
    let dir = site(name);
    let out = dir.join("dist");
    let report = crabbucket::build_with(&dir, &Ferrite, Options::new().out_dir(out.clone()))
        .unwrap_or_else(|err| panic!("Ferrite should build this site, but: {err}"));

    (report, out)
}

#[test]
fn the_design_system_the_book_builds_renders_a_whole_site() {
    let (report, out) = build("renders");

    assert_eq!(report.routes.len(), 3, "{:?}", report.routes);
    assert!(report.links >= 3, "nothing was checked: {}", report.links);
    assert!(out.join("guide/first/index.html").is_file());
    assert!(out.join("site.css").is_file());
}

#[test]
fn its_layouts_and_its_directive_both_do_something() {
    let (_, out) = build("layouts");

    let chapter = fs::read_to_string(out.join("guide/first/index.html")).expect("no chapter");

    // The docs layout lists the section, so the sibling is on the page.
    assert!(
        chapter.contains("/fleet/guide/"),
        "no section list: {chapter}"
    );
    // The callout the theme registers, rather than a literal `:::callout'.
    assert!(chapter.contains("fe-note"), "the callout did not render");
    assert!(!chapter.contains(":::"), "a directive was left as text");
    // `~/' resolved against the base path rather than being written through.
    assert!(
        chapter.contains("href=\"/fleet/\""),
        "`~/' was not resolved"
    );
}

#[test]
fn the_stylesheet_carries_the_tokens_and_the_namespace() {
    let (_, out) = build("stylesheet");

    let css = fs::read_to_string(out.join("site.css")).expect("no stylesheet");

    assert!(css.contains("--color-paper"), "no tokens in the stylesheet");
    assert!(css.contains(".fe-page"), "no namespaced classes");
    assert!(
        !css.contains(".&"),
        "a class root was left unexpanded: {css}"
    );
}
