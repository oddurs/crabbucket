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
//! The promises in `doc/STABILITY`, as far as a test can hold them.
//!
//! Most of that document is a commitment rather than a property -- nothing
//! here can stop somebody renaming a method.  What a test *can* do is hold the
//! three rules in section 7, which is where the last review found three
//! separate failures on the same day, and pin the decisions in section 5 so
//! that undoing one is deliberate.

use std::fs;
use std::path::Path;

/// The document itself, so the tests can check they still match it.
fn stability() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../doc/STABILITY");
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{}: {err}", path.display()))
        .replace("\r\n", "\n")
}

/// Every source file of this crate, with line endings normalised.
///
/// A checkout on Windows has CRLF, and a multi-line pattern written with `\n`
/// then matches nothing -- which is exactly the mistake the CRLF bug in
/// `content::split` was, made again one file away from the test that caught
/// it.
fn source() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut all = String::new();

    for entry in fs::read_dir(&dir).expect("no src") {
        let path = entry.expect("unreadable").path();
        if path.extension().is_some_and(|e| e == "rs") {
            all.push_str(&fs::read_to_string(&path).unwrap_or_default());
        }
    }

    all.replace("\r\n", "\n")
}

#[test]
fn rule_one_everything_a_caller_needs_is_at_the_crate_root() {
    // Section 7, rule 1.  `crabbucket::build` compiling while
    // `crabbucket::build_with` did not is how this rule was discovered.
    //
    // Naming these rather than deriving them is the point: adding a public
    // type a caller must touch should require saying so here.
    let _: fn(&Path, &Plain) -> crabbucket::Result<crabbucket::Report> = crabbucket::build;
    let _: fn(&Path, &Plain, crabbucket::Options) -> crabbucket::Result<crabbucket::Report> =
        crabbucket::build_with;

    // Constructed by a caller.
    let _ = crabbucket::Options::new();
    let _ = crabbucket::Directives::new();
    let _ = crabbucket::Data::default();
    let _ = crabbucket::StyleSheet::new();
    let _ = crabbucket::Style::new("ns", "name", ".& {}");
    let _ = crabbucket::Config::blank();

    // Received by a caller, and named in a signature or a match.  Naming the
    // type is the assertion; the function only exists to hold the names.
    fn nameable<T>() {}

    nameable::<crabbucket::Error>();
    nameable::<crabbucket::Report>();
    nameable::<crabbucket::Url>();
    nameable::<crabbucket::DeadLink>();
    nameable::<crabbucket::Reason>();
    nameable::<crabbucket::Snippet>();
    nameable::<crabbucket::Heading>();
    nameable::<crabbucket::Body>();
    nameable::<crabbucket::NavItem>();
    nameable::<crabbucket::SiteIndex>();
    nameable::<crabbucket::PageRef>();
    nameable::<crabbucket::FeedLink>();
    nameable::<crabbucket::Feed>();
    nameable::<crabbucket::NoExtra>();
    nameable::<crabbucket::Context<'_>>();
    nameable::<crabbucket::Collection<crabbucket::NoExtra>>();
    nameable::<crabbucket::Entry<crabbucket::NoExtra>>();
}

#[test]
fn rule_two_what_a_caller_receives_prints_usefully() {
    // Section 7, rule 2.  Display, not Debug, for anything a person reads.
    fn displays<T: std::fmt::Display>() {}

    displays::<crabbucket::Error>();
    displays::<crabbucket::Report>();
    displays::<crabbucket::Url>();
    displays::<crabbucket::DeadLink>();
    displays::<crabbucket::directive::Fault>();
}

#[test]
fn rule_three_a_report_says_everything_it_carries() {
    // Section 7, rule 3, and the reason it exists: a build with something to
    // say used to say it to nobody.
    let printed = format!("{}", sample_report());

    assert!(printed.contains("1 draft skipped"), "got {printed}");
    assert!(printed.contains("warning: something"), "got {printed}");
}

fn sample_report() -> crabbucket::Report {
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/drafts");
    let out = std::env::temp_dir().join(format!("crabbucket-stability-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let mut report = crabbucket::build_with(&site, &Plain, crabbucket::Options::new().out_dir(out))
        .expect("the drafts fixture should build");

    report.warnings.push("something".to_string());
    report
}

#[test]
fn section_five_the_types_that_grow_say_so() {
    // Deciding this later is itself the breaking change, so it is pinned.
    let source = source();

    for declaration in [
        "#[non_exhaustive]\n#[derive(Debug)]\npub enum Error {",
        "pub enum Reason {",
        "pub struct Options {",
        "pub struct Report {",
        "pub struct Config {",
    ] {
        let at = source
            .find(declaration)
            .unwrap_or_else(|| panic!("no {declaration}"));
        let before = &source[at.saturating_sub(120)..at + declaration.len()];

        assert!(
            before.contains("#[non_exhaustive]"),
            "`{declaration}` is no longer non_exhaustive, which doc/STABILITY section 5 promises"
        );
    }
}

#[test]
fn section_five_names_what_it_marks() {
    let document = stability();

    for named in ["Error, Reason", "Config, Options, Report"] {
        assert!(
            document.contains(named),
            "section 5 no longer names `{named}`"
        );
    }

    assert!(
        document.contains("Not marked, deliberately: `PageMeta'"),
        "section 5 no longer says what it leaves alone"
    );
}

#[test]
fn the_framework_class_names_are_the_ones_the_document_promises() {
    // Section 1: a design system styles these and may not rename them.
    assert_eq!(
        crabbucket::style::FRAMEWORK_CLASSES,
        ["heading-anchor", "code"]
    );
    assert_eq!(crabbucket::style::HIGHLIGHT_PREFIX, "tok-");

    let document = stability();
    assert!(
        document.contains("`heading-anchor', `code'"),
        "section 1 no longer names them"
    );
}

#[test]
fn the_document_still_says_what_it_does_not_promise() {
    // The half people skip, and the half that matters when they are annoyed.
    let document = stability();

    for section in [
        "2.  What is not covered",
        "3.  The awkward one",
        "6.  Deprecation",
    ] {
        assert!(document.contains(section), "doc/STABILITY lost `{section}`");
    }
}

// A design system, because several of these need one to build with.
use crabbucket::theme::{NoExtra, Page, Theme};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct Layout;

struct Plain;

impl Theme for Plain {
    type Layout = Layout;
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        format!(
            "<!doctype html><html><head><title>{}</title></head><body><main>{}</main></body></html>",
            page.meta.title, page.html
        )
    }

    fn stylesheet(&self) -> String {
        String::new()
    }
}
