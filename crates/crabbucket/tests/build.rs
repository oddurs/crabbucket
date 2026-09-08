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
//! End-to-end tests for the build, one fixture site per gate.
//!
//! The gates are the product.  "The build either fails, or the site is
//! correct" is a claim about failure, and a claim about failure that is not
//! tested is a claim that will quietly stop being true -- so these assert that
//! a bad site *fails*, and that it fails with a message naming the file.
//!
//! The theme here is defined in this file rather than borrowed from
//! `crabbucket-ui`, both because the dependency would run the wrong way and
//! because it keeps `Theme` honest: if the trait cannot be implemented from
//! outside in thirty lines, that is worth finding out here.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crabbucket::error::Error;
use crabbucket::links::Reason;
use crabbucket::site::{Options, Report};
use crabbucket::theme::{Page, Theme};
use serde::Deserialize;

/// The layouts the test theme offers, so `unknown-layout` has something to be
/// unknown to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Layout {
    #[default]
    Page,
    Docs,
}

struct Plain;

impl Theme for Plain {
    type Layout = Layout;

    fn render(&self, page: &Page<'_, Layout>) -> String {
        let nav = page
            .nav()
            .iter()
            .map(|item| format!("<a href=\"{}\">{}</a>", item.href, item.label))
            .collect::<String>();

        format!(
            "<!doctype html><html><head><title>{}</title>\
             <link rel=\"stylesheet\" href=\"{}\"></head>\
             <body><nav>{nav}</nav><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.html,
        )
    }

    fn stylesheet(&self) -> String {
        "body { color: inherit; }".to_string()
    }

    fn router_js(&self) -> String {
        "/* test */".to_string()
    }
}

/// Builds a fixture into a scratch directory of its own.
///
/// Output goes somewhere else entirely, so the fixtures stay clean, the suite
/// is safe to run in parallel, and a failing test leaves nothing untracked in
/// the repository.
fn build(fixture: &str) -> (Result<Report, Error>, PathBuf) {
    build_with(fixture, None)
}

/// Builds a fixture, optionally overriding the base path.
///
/// The output directory is unique per call rather than per fixture: two tests
/// may build the same fixture, and the suite runs them at the same time.
fn build_with(fixture: &str, base: Option<&str>) -> (Result<Report, Error>, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let site = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sites")
        .join(fixture);
    let out = std::env::temp_dir().join(format!(
        "crabbucket-test-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        fixture
    ));

    let _ = fs::remove_dir_all(&out);
    let options = Options {
        out_dir: Some(out.clone()),
        base: base.map(|base| base.to_string()),
    };

    (crabbucket::site::build_with(&site, &Plain, &options), out)
}

/// Builds a fixture that is expected to succeed.
fn ok(fixture: &str) -> (Report, PathBuf) {
    let (result, out) = build(fixture);
    match result {
        Ok(report) => (report, out),
        Err(err) => panic!("fixture `{fixture}` should build, but: {err}"),
    }
}

/// Builds a fixture that is expected to fail.
fn fails(fixture: &str) -> Error {
    let (result, _) = build(fixture);
    match result {
        Err(err) => err,
        Ok(report) => panic!(
            "fixture `{fixture}` should fail, but wrote {:?}",
            report.routes
        ),
    }
}

fn read(out: &Path, path: &str) -> String {
    fs::read_to_string(out.join(path))
        .unwrap_or_else(|err| panic!("{}: {err}", out.join(path).display()))
}

// ---------------------------------------------------------------- it builds

#[test]
fn a_good_site_builds_into_directories_of_index_files() {
    let (report, out) = ok("ok");

    assert_eq!(report.routes, ["", "docs"]);
    assert_eq!(report.drafts, 0);
    assert!(report.links > 0, "no links were checked");

    assert!(out.join("index.html").is_file());
    assert!(out.join("docs/index.html").is_file());
    assert!(out.join("site.css").is_file());
    assert!(
        out.join("router.js").is_file(),
        "the site asked for the router"
    );
}

#[test]
fn the_nojekyll_file_is_written_because_pages_needs_it() {
    let (_, out) = ok("ok");
    assert!(out.join(".nojekyll").is_file());
}

#[test]
fn static_files_are_copied_and_are_linkable() {
    let (_, out) = ok("ok");
    assert_eq!(read(&out, "logo.txt"), "x");
}

#[test]
fn a_url_gets_a_sitemap_and_robots_and_nothing_else_does() {
    let (_, out) = ok("ok");
    let sitemap = read(&out, "sitemap.xml");
    assert!(sitemap.contains("<loc>https://example.com/</loc>"));
    assert!(sitemap.contains("<loc>https://example.com/docs/</loc>"));
    assert!(read(&out, "robots.txt").contains("Sitemap: https://example.com/sitemap.xml"));

    let (_, bare) = ok("drafts");
    assert!(!bare.join("sitemap.xml").exists(), "no url, so no sitemap");
    assert!(
        !bare.join("robots.txt").exists(),
        "no url, so no robots.txt"
    );
}

#[test]
fn nested_content_keeps_its_shape_on_any_platform() {
    let (report, out) = ok("nested");
    assert_eq!(report.routes, ["", "a/b/c"]);
    assert!(out.join("a/b/c/index.html").is_file());
}

#[test]
fn a_draft_is_skipped_and_leaves_the_navigation() {
    let (report, out) = ok("drafts");

    assert_eq!(report.routes, [""]);
    assert_eq!(report.drafts, 1);
    assert!(!out.join("secret").exists(), "a draft was written anyway");
    assert!(
        !read(&out, "index.html").contains("Secret"),
        "a draft reached the navigation"
    );
}

// ------------------------------------------------------------- the base path

#[test]
fn every_link_carries_the_base_path_exactly_once() {
    let (_, out) = ok("base-path");
    let html = read(&out, "index.html");

    for href in ["/repo/", "/repo/docs/", "/repo/site.css"] {
        assert!(
            html.contains(&format!("\"{href}\"")),
            "{href} is missing from {html}"
        );
    }

    assert!(
        !html.contains("/repo/repo/"),
        "the base path was applied twice"
    );
    assert!(!html.contains("\"/docs/\""), "a link escaped the base path");
}

#[test]
fn an_override_reaches_every_link_without_touching_the_config() {
    // `elsewhere`, unslashed, to check that an override normalises the same
    // way the configuration file does.
    let (result, out) = build_with("base-path", Some("elsewhere"));
    result.expect("should build");

    let html = read(&out, "index.html");

    // Every link the framework generated moved.
    assert!(html.contains("\"/elsewhere/docs/\""), "got {html}");
    assert!(html.contains("\"/elsewhere/site.css\""), "got {html}");
    assert!(
        !html.contains("\"/repo/site.css\""),
        "the configured base survived"
    );

    // The one link the *content* hard-codes did not, and is no longer checked,
    // because a site-absolute link outside the base may well belong to
    // something else served from the same domain.
    assert!(
        html.contains("/repo/docs/"),
        "content is not rewritten, and should not be"
    );
}

// --------------------------------------------------------------- it refuses

#[test]
fn a_page_with_no_title_fails_and_names_the_file() {
    let err = fails("missing-title");
    assert!(matches!(err, Error::Schema { .. }));

    let message = err.render(false);
    assert!(
        message.contains("missing-title/content/index.md"),
        "got {message}"
    );
    assert!(message.contains("missing field `title`"), "got {message}");
}

#[test]
fn a_schema_error_shows_the_offending_line_with_a_caret() {
    let message = fails("unknown-layout").render(false);
    let lines: Vec<&str> = message.lines().collect();

    assert!(
        lines[0].contains(":3:10:"),
        "expected file:line:col, got {}",
        lines[0]
    );
    assert!(
        lines[0].contains("unknown variant `dcos`"),
        "got {}",
        lines[0]
    );
    assert!(
        lines[0].contains("`page`") && lines[0].contains("`docs`"),
        "got {}",
        lines[0]
    );
    assert_eq!(lines[2].trim_start(), "3 | layout = \"dcos\"");
    assert!(lines[3].contains('^'), "no caret in {}", lines[3]);
}

#[test]
fn a_file_with_no_frontmatter_fails() {
    let err = fails("no-frontmatter");
    assert!(matches!(err, Error::MissingFrontmatter { .. }));
    assert!(
        err.render(false)
            .contains("no-frontmatter/content/index.md")
    );
}

#[test]
fn an_unterminated_frontmatter_block_fails() {
    let err = fails("unterminated");
    assert!(matches!(err, Error::UnterminatedFrontmatter { .. }));
    assert!(err.render(false).contains("unterminated/content/index.md"));
}

#[test]
fn a_link_to_a_route_that_does_not_exist_fails() {
    let Error::DeadLinks(dead) = fails("dead-route") else {
        panic!("expected dead links");
    };

    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, Reason::NoSuchRoute);
    assert_eq!(dead[0].target, "/gone/");
    assert!(dead[0].source.ends_with("content/index.md"));
}

#[test]
fn a_link_to_an_asset_that_does_not_exist_fails_as_an_asset() {
    let Error::DeadLinks(dead) = fails("dead-asset") else {
        panic!("expected dead links");
    };

    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, Reason::NoSuchAsset);
    assert!(
        dead[0].describe().contains("not a file"),
        "got {}",
        dead[0].describe()
    );
}

#[test]
fn a_fragment_naming_a_heading_that_does_not_exist_fails() {
    let Error::DeadLinks(dead) = fails("dead-anchor") else {
        panic!("expected dead links");
    };

    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, Reason::NoSuchAnchor);
    assert_eq!(dead[0].href, "#uninstalling");
}

#[test]
fn a_relative_link_on_the_error_page_fails() {
    let Error::DeadLinks(dead) = fails("relative-on-error-page") else {
        panic!("expected dead links");
    };

    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, Reason::RelativeOnErrorPage);
    assert!(dead[0].source.ends_with("404.md"));
}

#[test]
fn every_failure_names_a_file() {
    for fixture in [
        "missing-title",
        "no-frontmatter",
        "unterminated",
        "unknown-layout",
        "dead-route",
        "dead-asset",
        "dead-anchor",
        "relative-on-error-page",
    ] {
        let err = build(fixture).0.expect_err("should fail");
        assert!(
            err.path().is_some(),
            "`{fixture}` produced an error with no file"
        );
    }
}
