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
//! The same site, built by a design system that is not the default one.
//!
//! This is the whole point of the crate.  `crabbucket-ui` and Plain disagree
//! about layouts, colours, directives and whether to ship JavaScript at all,
//! and neither of them is named anywhere in `crabbucket`.  If that stops being
//! true, this fails.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crabbucket::site::Options;
use crabbucket::theme::Theme;
use crabbucket_theme_plain::{Plain, tok};

/// crabbucket's own site, which was written for the other design system.
fn example_site() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/site")
}

fn build_into(name: &str) -> (crabbucket::Report, PathBuf) {
    let out = std::env::temp_dir().join(format!("crabbucket-plain-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    // The site's own base, not an override: the error page carries a
    // site-absolute link, and content has no way to write one that survives
    // the base changing under it.
    let options = Options {
        out_dir: Some(out.clone()),
        base: None,
    };
    let report = crabbucket::build_with(&example_site(), &Plain, &options)
        .unwrap_or_else(|err| panic!("Plain cannot build the example site: {err}"));

    (report, out)
}

#[test]
fn a_site_written_for_another_design_system_still_builds() {
    let (report, out) = build_into("site");

    assert!(
        report.routes.len() > 5,
        "only {} routes",
        report.routes.len()
    );
    assert!(out.join("index.html").is_file());
    assert!(out.join("site.css").is_file());

    let html = fs::read_to_string(out.join("index.html")).expect("no index");
    assert!(
        html.contains("class=\"pl-page\""),
        "got the wrong design system: {html}"
    );
    assert!(
        !html.contains("cb-"),
        "a class from the other design system leaked: {html}"
    );
}

#[test]
fn a_design_system_that_declines_the_router_gets_neither_the_file_nor_a_tag() {
    // The example site's configuration asks for the router and for search.
    // Plain has neither, so it should be told once and shipped a page that
    // does not point at files nobody wrote.  A dangling script tag would fail
    // the link check before it reached this assertion.
    let (report, out) = build_into("declines");

    assert!(!out.join("router.js").exists());
    assert!(!out.join("search.js").exists());
    assert!(!out.join("search.json").exists());

    let html = fs::read_to_string(out.join("index.html")).expect("no index");
    assert!(
        !html.contains("<script"),
        "Plain ships no JavaScript: {html}"
    );

    assert_eq!(report.warnings.len(), 2, "got {:?}", report.warnings);
    assert!(report.warnings.iter().any(|w| w.contains("router")));
    assert!(report.warnings.iter().any(|w| w.contains("search")));
}

#[test]
fn its_own_tokens_come_from_its_own_file() {
    // A design system owns its palette.  Reaching into crabbucket's would make
    // every theme a fork of the default one.
    assert_eq!(
        tok::SCHEME,
        "light",
        "Plain is a document, and documents are light"
    );
    assert!(
        tok::color::PAPER.starts_with("var(--color-paper,"),
        "got {}",
        tok::color::PAPER
    );

    let base: BTreeMap<_, _> = tok::BASE.iter().copied().collect();
    assert!(base.contains_key("--color-ink"));
    assert!(
        !base.contains_key("--color-surface"),
        "that is the other design system's token"
    );
}

#[test]
fn one_layout_is_a_complete_answer() {
    // Three layouts and one layout give the same guarantee: a page asking for
    // something else fails the build.  This is the shorter list to read.
    let stylesheet = Plain.stylesheet();

    assert!(stylesheet.contains(".pl-page"), "got {stylesheet}");
    assert!(
        !stylesheet.contains("--docs"),
        "Plain has no docs layout: {stylesheet}"
    );
}

#[test]
fn the_defaults_carry_everything_this_design_system_does_not_want() {
    // Plain says nothing about the router or search, and the trait defaults
    // mean nothing about them reaches the output.  That is what makes writing
    // a second design system cheap.
    assert!(Plain.router_js().is_none());
    assert!(Plain.search_js().is_none());
}

#[test]
fn it_answers_to_the_component_names_the_content_uses() {
    // The other half of portability.  A page's `:::callout` names a
    // component, so a design system that cannot answer to the name cannot
    // render the page -- however different it chooses to look.
    assert_eq!(
        Plain.directives().names(),
        ["callout", "card", "cards", "step", "steps", "tab", "tabs"]
    );
}

#[test]
fn it_answers_to_the_layout_names_the_content_uses() {
    let (_, out) = build_into("layouts");

    // examples/site has pages saying `layout = "docs"` and `layout =
    // "landing"`, which Plain does not distinguish.  It says so with two
    // serde aliases rather than by failing.
    assert!(
        out.join("docs/routing/index.html").is_file(),
        "a docs page did not render"
    );
    assert!(
        out.join("index.html").is_file(),
        "the landing page did not render"
    );
}
