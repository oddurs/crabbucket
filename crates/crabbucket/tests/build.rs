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
use std::time::Duration;

use crabbucket::directive::Directives;
use crabbucket::error::Error;
use crabbucket::links::Reason;
use crabbucket::theme::{NoExtra, Page, Theme};
use crabbucket::{Options, Report};
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

/// The one directive the fixtures use, with a required attribute so that a
/// misspelled one has something to fail against.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Note {
    label: String,
}

struct Plain;

impl Theme for Plain {
    type Layout = Layout;
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        let nav = page
            .nav()
            .iter()
            .map(|item| format!("<a href=\"{}\">{}</a>", item.href, item.label))
            .collect::<String>();

        // A design system has to load the clients the site asked for.  Leaving
        // one out ships a feature that silently does nothing, so the build
        // says so -- see `Forgetful` below.
        let scripts = [
            page.config.router.then_some("router.js"),
            page.config.search.then_some("search.js"),
        ]
        .into_iter()
        .flatten()
        .map(|file| {
            format!(
                "<script defer src=\"{}\"></script>",
                crabbucket::Url::asset(page.config, file)
            )
        })
        .collect::<String>();

        format!(
            "<!doctype html><html><head><title>{}</title>\
             <link rel=\"stylesheet\" href=\"{}\">{scripts}</head>\
             <body><nav>{nav}</nav><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.html,
        )
    }

    fn directives(&self) -> Directives {
        let mut directives = Directives::new();
        directives.add("note", |props: Note, body| {
            maud::html! { aside data-label=(props.label) { (body) } }
        });
        directives
    }

    fn stylesheet(&self) -> String {
        "body { color: inherit; }".to_string()
    }

    fn router_js(&self) -> Option<String> {
        Some("/* test */".to_string())
    }

    fn search_js(&self) -> Option<String> {
        Some("/* test */".to_string())
    }
}

/// A design system that offers a router and then forgets to load it.
///
/// This is a real mistake made in a real design system: `search = true` wrote
/// `search.js`, the head had no tag for it, and the search box sat hidden
/// waiting for a client that never arrived.  Nothing was broken enough to
/// fail; it simply did nothing.
struct Forgetful;

impl Theme for Forgetful {
    type Layout = Layout;
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        format!(
            "<!doctype html><html><head><title>{}</title>\
             <link rel=\"stylesheet\" href=\"{}\"></head><body><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.html
        )
    }

    fn stylesheet(&self) -> String {
        String::new()
    }

    fn router_js(&self) -> Option<String> {
        Some("/* never loaded */".to_string())
    }
}

/// A design system that draws social cards.
///
/// The bytes are not a real image; this is about the plumbing -- where the
/// file lands, whether it is registered as an asset, and whether a site with
/// no absolute URL is asked for one at all.
struct Illustrated;

impl Theme for Illustrated {
    type Layout = Layout;
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        let card = page
            .card
            .map(|url| format!("<meta property=\"og:image\" content=\"{url}\">"))
            .unwrap_or_default();

        format!(
            "<!doctype html><html><head><title>{}</title>{card}\
             <link rel=\"stylesheet\" href=\"{}\"></head><body><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.html
        )
    }

    fn stylesheet(&self) -> String {
        String::new()
    }

    fn og_image(&self, page: &Page<'_, Self>) -> Option<Vec<u8>> {
        // Proof that the design system is given somewhere to keep things.
        assert!(
            page.cache.ends_with(".crabbucket"),
            "no cache directory: {:?}",
            page.cache
        );

        Some(format!("not-a-png:{}", page.route).into_bytes())
    }
}

/// The frontmatter a design system can ask a site for.
///
/// Required, deliberately: the interesting case is not reading a field, it is
/// what happens to a page that does not have one.
#[derive(Debug, Default, Deserialize)]
struct Summary {
    summary: String,
}

/// A design system that reads a field of its own.
struct Demanding;

impl Theme for Demanding {
    type Layout = Layout;
    type Extra = Summary;

    fn render(&self, page: &Page<'_, Self>) -> String {
        format!(
            "<!doctype html><html><head><title>{}</title>\
             <link rel=\"stylesheet\" href=\"{}\"></head>\
             <body><p id=\"summary\">{}</p><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.meta.extra.summary,
            page.html
        )
    }

    fn stylesheet(&self) -> String {
        String::new()
    }
}

/// A design system with no router and no search, for the fixtures that ask
/// for one anyway.
struct Bare;

impl Theme for Bare {
    type Layout = Layout;
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        format!(
            "<!doctype html><html><head><title>{}</title><link rel=\"stylesheet\" href=\"{}\"></head><body><main>{}</main></body></html>",
            page.meta.title,
            crabbucket::Url::asset(page.config, "site.css"),
            page.html
        )
    }

    fn stylesheet(&self) -> String {
        "body { color: inherit; }".to_string()
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

/// Builds a fixture with a design system other than the usual one.
fn build_theme<T: Theme>(fixture: &str, theme: &T) -> (Report, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let site = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sites")
        .join(fixture);
    let out = std::env::temp_dir().join(format!(
        "crabbucket-theme-{}-{}-{fixture}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let _ = fs::remove_dir_all(&out);
    let options = Options::new().out_dir(out.clone());

    let report = crabbucket::build_with(&site, theme, options)
        .unwrap_or_else(|err| panic!("`{fixture}` should build, but: {err}"));

    (report, out)
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
    let options = Options::new().out_dir(out.clone()).maybe_base(base);

    (crabbucket::build_with(&site, &Plain, options), out)
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
fn what_the_build_writes_has_the_same_line_endings_everywhere() {
    // A design system's stylesheet and clients arrive through `include_str!`,
    // so on a CRLF checkout they carry carriage returns.  The same site built
    // on two machines should be the same bytes.
    let (_, out) = ok("ok");

    for file in ["index.html", "site.css", "router.js"] {
        let written = read(&out, file);
        assert!(!written.contains('\r'), "{file} was written with CRLF");
    }
}

#[test]
fn a_content_file_with_windows_line_endings_builds() {
    // Git hands a checkout CRLF on Windows, so this is what a content file
    // looks like to half the world.  It did not parse at all until the
    // platform matrix said so.
    let (report, out) = ok("crlf");

    assert_eq!(report.routes, [""]);

    let html = read(&out, "index.html");
    assert!(html.contains("Written on Windows"), "got {html}");
    assert!(
        html.contains("carriage return"),
        "the body did not survive: {html}"
    );
}

#[test]
fn nested_content_keeps_its_shape_on_any_platform() {
    // Routes are built from forward slashes and paths from the platform's
    // separator.  Nothing on a Unix machine notices the difference, which is
    // why the matrix runs this on Windows.
    let (report, out) = ok("nested");

    assert_eq!(report.routes, ["", "a/b/c", "a/b/c/d/deeper"]);
    assert!(out.join("a/b/c/index.html").is_file());
    assert!(
        out.join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("deeper")
            .join("index.html")
            .is_file()
    );

    // And the links out of the deep page resolved, or the build would have
    // failed before here.
    let html = read(&out, "a/b/c/d/deeper/index.html");
    assert!(
        html.contains("href=\"/\""),
        "the site-root link did not resolve: {html}"
    );
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

// -------------------------------------------------------------- directives

#[test]
fn a_directive_renders_through_its_component() {
    let (_, out) = ok("directives");
    let html = read(&out, "index.html");

    assert!(
        html.contains("<aside data-label=\"Careful\">"),
        "got {html}"
    );
    assert!(
        html.contains("<aside data-label=\"Nested\">"),
        "directives do not nest: {html}"
    );
    assert!(
        html.contains("<a href=\"docs/\">link</a>"),
        "Markdown inside a directive: {html}"
    );
}

#[test]
fn a_directive_inside_a_code_fence_reaches_the_page_as_text() {
    let (_, out) = ok("directives");
    let html = read(&out, "index.html");

    assert!(
        html.contains("not a directive"),
        "the fenced example vanished"
    );
    assert!(
        !html.contains("data-label=\"not a directive\""),
        "the fence was rendered: {html}"
    );
}

#[test]
fn headings_inside_and_outside_directives_share_one_id_space() {
    let (report, _) = ok("directives");
    assert_eq!(report.routes, ["", "docs"]);
}

#[test]
fn a_link_written_inside_a_directive_is_still_checked() {
    // The fixture links to `docs/` from inside a directive, and `docs/` exists.
    // Removing it would make this build fail, which is the point.
    let (report, _) = ok("directives");
    assert!(report.links >= 2, "links inside directives were not seen");
}

#[test]
fn an_unknown_directive_fails_and_lists_the_ones_that_exist() {
    let message = fails("unknown-directive").render(false);
    assert!(
        message.contains("unknown directive `callout`"),
        "got {message}"
    );
    assert!(message.contains("`note`"), "got {message}");
    assert!(
        message.contains(":7:1:"),
        "the directive opens on line 7: {message}"
    );
}

#[test]
fn a_misspelled_attribute_fails_as_serde_sees_it() {
    let message = fails("bad-attribute").render(false);
    assert!(message.contains("unknown field `lable`"), "got {message}");
    assert!(message.contains(":7:1:"), "got {message}");
}

#[test]
fn an_unterminated_directive_fails_at_the_line_it_opened_on() {
    let message = fails("unterminated-directive").render(false);
    assert!(
        message.contains("unterminated directive `note`"),
        "got {message}"
    );
    assert!(message.contains(":7:1:"), "got {message}");
}

// ------------------------------------------------- what a theme declines

#[test]
fn a_theme_without_a_router_gets_a_warning_and_a_working_site() {
    // The site asked for a router.  This design system has none.  That is
    // worth saying and not worth stopping for: the page still works, it just
    // has less in it than the configuration implies.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/ok");
    let out = std::env::temp_dir().join(format!("crabbucket-bare-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out.clone());
    let report = crabbucket::build_with(&site, &Bare, options).expect("should still build");

    assert!(
        !out.join("router.js").exists(),
        "a client was written for a theme with none"
    );
    assert!(out.join("index.html").is_file(), "the site did not build");

    assert_eq!(report.warnings.len(), 1, "got {:?}", report.warnings);
    assert!(
        report.warnings[0].contains("asks for the router"),
        "got {:?}",
        report.warnings
    );
    assert!(
        report.warnings[0].contains("has none"),
        "got {:?}",
        report.warnings
    );
}

#[test]
fn a_theme_without_search_writes_no_index_either() {
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/search");
    let out = std::env::temp_dir().join(format!("crabbucket-bare-search-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out.clone());
    let report = crabbucket::build_with(&site, &Bare, options).expect("should still build");

    assert!(
        !out.join("search.json").exists(),
        "an index with no client to read it"
    );
    assert!(!out.join("search.js").exists());
    assert!(
        report.warnings[0].contains("asks for the search"),
        "got {:?}",
        report.warnings
    );
}

#[test]
fn a_client_nobody_loads_is_reported() {
    // The opposite of a dead link, and just as broken: the feature is
    // configured, the file is shipped, and nothing happens.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/ok");
    let out = std::env::temp_dir().join(format!("crabbucket-forgot-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out.clone());
    let report = crabbucket::build_with(&site, &Forgetful, options).expect("should still build");

    assert!(out.join("router.js").is_file(), "the client was written");

    assert_eq!(report.warnings.len(), 1, "got {:?}", report.warnings);
    assert!(
        report.warnings[0].contains("router.js"),
        "got {:?}",
        report.warnings
    );
    assert!(
        report.warnings[0].contains("no page loads it"),
        "got {:?}",
        report.warnings
    );
}

// ------------------------------------------------------------------ report

#[test]
fn the_whole_api_a_caller_needs_is_at_the_crate_root() {
    // `build` was re-exported and `build_with` was not, so the obvious
    // symmetric name did not compile and the workaround was to move `dist/`
    // by hand.  This test is here so that cannot happen again.
    let _: fn(&Path, &Plain) -> Result<Report, Error> = crabbucket::build;
    let _: fn(&Path, &Plain, Options) -> Result<Report, Error> = crabbucket::build_with;
    let _ = crabbucket::Options::default();
}

#[test]
fn printing_a_report_says_everything_it_has_to_say() {
    let (report, _) = ok("drafts");

    // Whatever anybody writes first has to be the correct thing, because a
    // warning printed to nobody is a warning that was not printed.
    let printed = report.to_string();

    assert!(printed.contains("1 page, "), "got {printed}");
    assert!(printed.contains("links checked -> "), "got {printed}");
    assert!(
        printed.contains("1 draft skipped"),
        "the draft went unmentioned: {printed}"
    );
}

#[test]
fn a_report_pluralises_so_that_nobody_downstream_has_to() {
    let (one, _) = ok("drafts");
    assert!(
        one.headline().starts_with("1 page,"),
        "got {}",
        one.headline()
    );
    assert_eq!(one.drafts_line().as_deref(), Some("1 draft skipped"));

    let (many, _) = ok("ok");
    assert!(
        many.headline().starts_with("2 pages,"),
        "got {}",
        many.headline()
    );
    assert_eq!(many.drafts_line(), None, "no drafts, so no line about them");
}

#[test]
fn a_warning_reaches_the_printed_report() {
    let mut report = ok("ok").0;
    report.warnings.push("the sky is falling".to_string());

    assert!(
        report.to_string().contains("warning: the sky is falling"),
        "got {report}"
    );
}

// ------------------------------------------------------------------ search

#[test]
fn asking_for_search_writes_an_index_and_a_client() {
    let (report, out) = ok("search");

    assert!(out.join("search.js").is_file(), "no client");
    assert!(
        report.warnings.is_empty(),
        "unexpected warnings: {:?}",
        report.warnings
    );

    let index = read(&out, "search.json");
    assert!(index.starts_with('['), "got {index}");
    assert!(
        index.contains("\"u\":\"/repo/\""),
        "the url carries the base path: {index}"
    );
    assert!(index.contains("\"t\":\"Home\""), "got {index}");
    assert!(index.contains("a-heading"), "headings are indexed: {index}");
    assert!(
        index.contains("searchable words"),
        "body text is indexed: {index}"
    );
}

#[test]
fn the_index_is_valid_json_including_the_awkward_characters() {
    let (_, out) = ok("search");
    let index = read(&out, "search.json");

    // The fixture contains a quote and a backslash on purpose.
    assert!(index.contains("\\\""), "the quote was not escaped: {index}");
    assert!(
        index.contains("\\\\"),
        "the backslash was not escaped: {index}"
    );

    // Parsing it as TOML's JSON-compatible value is not available, so check
    // the one property that hand-written JSON gets wrong: balanced quoting.
    let unescaped = index.replace("\\\\", "").replace("\\\"", "");
    assert_eq!(
        unescaped.matches('"').count() % 2,
        0,
        "unbalanced quotes: {index}"
    );
}

#[test]
fn the_error_page_is_not_in_the_search_index() {
    let (_, out) = ok("search");
    let index = read(&out, "search.json");

    assert!(
        !index.contains("Not found"),
        "the 404 is searchable: {index}"
    );
}

#[test]
fn a_site_that_did_not_ask_for_search_gets_none() {
    let (_, out) = ok("ok");
    assert!(!out.join("search.json").exists());
    assert!(!out.join("search.js").exists());
}

// ------------------------------------------------- directives the site owns

#[derive(Debug, Deserialize)]
struct Things {
    #[serde(flatten)]
    by_name: std::collections::BTreeMap<String, Thing>,
}

#[derive(Debug, Deserialize)]
struct Thing {
    label: String,
}

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

/// The site's own directive, reading the site's own data.
fn site_directives() -> Directives {
    let mut directives = Directives::new();

    directives.add_with("thing", |props: Named, _, context| {
        let things: Things = context.data("things")?;
        let thing = things
            .by_name
            .get(&props.name)
            .ok_or_else(|| format!("there is no thing called `{}`", props.name))?;

        Ok(maud::html! { p."thing" { (thing.label) } })
    });

    directives
}

#[test]
fn a_site_can_register_a_directive_of_its_own() {
    // A component usually belongs to a design system.  Not always: this one
    // reads data the site generates, and no design system should know that.
    let (result, out) = build_owning("site-data", site_directives());
    result.expect("should build");

    let html = read(&out, "index.html");
    assert!(
        html.contains("The first thing"),
        "the directive did not run: {html}"
    );
}

#[test]
fn a_site_directive_that_cannot_find_its_data_fails_at_the_line() {
    let mut directives = Directives::new();
    directives.add_with("thing", |_: Named, _, context| {
        let _: Things = context.data("absent")?;
        Ok(maud::html! {})
    });

    let (result, _) = build_owning("site-data", directives);
    let message = result.expect_err("should fail").render(false);

    assert!(
        names(&message, "content/index.md") && message.contains(":7:1"),
        "got {message}"
    );
    assert!(message.contains("no `data/absent.toml`"), "got {message}");
}

#[test]
fn a_site_may_not_quietly_replace_a_component() {
    // Overriding a design system's component would change every page that
    // uses it without a word.
    let mut directives = Directives::new();
    directives.add("note", |_: Named, body| body);

    let (result, _) = build_owning("directives", directives);
    let message = result.expect_err("should fail").render(false);

    assert!(
        message.contains("`note` is registered by both"),
        "got {message}"
    );
}

// ---------------------------------------------- pages the site renders itself

/// Whether a rendered message names a path.
///
/// A path renders with the platform's separator, so comparing one against a
/// literal with forward slashes passes everywhere except Windows -- which is
/// exactly the kind of thing a single-platform test suite never notices.
fn names(message: &str, path: &str) -> bool {
    let native = path
        .split('/')
        .collect::<Vec<_>>()
        .join(std::path::MAIN_SEPARATOR_STR);

    message.contains(&native)
}

/// Builds a fixture with directives the site brought.
fn build_owning(fixture: &str, directives: Directives) -> (Result<Report, Error>, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let site = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sites")
        .join(fixture);
    let out = std::env::temp_dir().join(format!(
        "crabbucket-owning-{}-{}-{fixture}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out.clone()).directives(directives);

    (crabbucket::build_with(&site, &Plain, options), out)
}

/// Builds a fixture with bodies for the pages it declares.
fn build_declaring(fixture: &str, pages: &[(&str, &str)]) -> (Result<Report, Error>, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let site = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sites")
        .join(fixture);
    let out = std::env::temp_dir().join(format!(
        "crabbucket-declared-{}-{}-{fixture}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out.clone()).pages(
        pages
            .iter()
            .map(|(route, body)| (route.to_string(), body.to_string()))
            .collect(),
    );

    (crabbucket::build_with(&site, &Plain, options), out)
}

#[test]
fn a_page_the_site_rendered_is_a_page_like_any_other() {
    let (result, out) = build_declaring("declared", &[("made-up", "<p>from Rust</p>")]);
    let report = result.expect("should build");

    assert!(
        report.routes.contains(&"made-up".to_string()),
        "got {:?}",
        report.routes
    );

    let html = read(&out, "made-up/index.html");
    assert!(
        html.contains("<p>from Rust</p>"),
        "the body did not reach the page: {html}"
    );
    assert!(
        html.contains("<title>Made up"),
        "the declared title was not used: {html}"
    );

    // In the navigation, because it declared a nav_order.
    let other = read(&out, "other/index.html");
    assert!(other.contains("made-up/"), "not in the navigation: {other}");
}

#[test]
fn a_declared_page_is_link_checked_like_any_other() {
    let (result, _) = build_declaring("declared", &[("made-up", r#"<a href="/nowhere/">x</a>"#)]);

    let Err(Error::DeadLinks(dead)) = result else {
        panic!("a dead link in a rendered page should fail");
    };

    assert_eq!(dead.len(), 1);
    assert!(
        dead[0].source.ends_with("site.toml"),
        "it names where the page was declared"
    );
}

#[test]
fn declaring_a_page_and_not_rendering_it_fails() {
    let (result, _) = build_declaring("unrendered", &[]);
    let err = result.expect_err("should fail");

    assert!(matches!(err, Error::Unrendered { .. }));
    assert!(
        err.render(false).contains("`promised` is declared"),
        "got {}",
        err.render(false)
    );
}

#[test]
fn rendering_a_page_and_not_declaring_it_fails() {
    // The reverse, and just as important: a page with a body and no title,
    // no layout and no place in the navigation is not a page.
    let (result, _) = build_declaring(
        "declared",
        &[
            ("made-up", "<p>fine</p>"),
            ("surprise", "<p>not declared</p>"),
        ],
    );

    let err = result.expect_err("should fail");
    assert!(matches!(err, Error::Undeclared { .. }));
    assert!(
        err.render(false).contains("`surprise`"),
        "got {}",
        err.render(false)
    );
}

#[test]
fn declaring_a_page_at_a_route_the_content_already_has_fails() {
    let (result, _) = build_declaring("collides", &[("other", "<p>x</p>")]);
    let err = result.expect_err("should fail");

    assert!(matches!(err, Error::Collides { .. }));
    assert!(
        err.render(false).contains("`other`"),
        "got {}",
        err.render(false)
    );
}

#[test]
fn a_declared_page_gets_the_same_frontmatter_checking() {
    // `layout` and any extra fields are the design system's types, so a
    // declared page is checked exactly as a written one is.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/declared");
    let out = std::env::temp_dir().join(format!("crabbucket-declared-meta-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out).page("made-up", "<p>x</p>");

    // `Demanding` requires a `summary`, and the declaration has none.
    let err = crabbucket::build_with(&site, &Demanding, options).expect_err("should fail");
    let message = err.render(false);

    assert!(
        message.contains("site.toml"),
        "it names where the page was declared: {message}"
    );
    assert!(message.contains("missing field `summary`"), "got {message}");
}

// -------------------------------------------------- frontmatter a site adds

#[test]
fn a_design_system_reads_the_frontmatter_it_declares() {
    let (_, out) = build_theme("extra", &Demanding);
    let html = read(&out, "index.html");

    assert!(
        html.contains("A sentence the design system asked for."),
        "the field did not reach the page: {html}"
    );
}

#[test]
fn a_page_missing_a_field_the_design_system_requires_fails() {
    // The whole point.  Before this, a site could carry a field and the
    // framework would drop it without a word.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/missing-extra");
    let out = std::env::temp_dir().join(format!("crabbucket-extra-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let options = Options::new().out_dir(out);
    let err = crabbucket::build_with(&site, &Demanding, options).expect_err("should fail");

    let message = err.render(false);
    assert!(
        names(&message, "missing-extra/content/index.md"),
        "got {message}"
    );
    assert!(message.contains("missing field `summary`"), "got {message}");
}

#[test]
fn a_design_system_that_declares_nothing_is_unaffected() {
    // `Plain` reads no extra frontmatter, and the fixture has two fields it
    // has never heard of.  It builds.
    let (_, out) = build_theme("extra", &Plain);
    assert!(out.join("index.html").is_file());
}

#[test]
fn a_key_nothing_declares_is_still_ignored_silently() {
    // Honest limitation.  serde cannot combine `flatten` with
    // `deny_unknown_fields`, so a misspelled frontmatter key still vanishes
    // rather than failing.  Asserted so the day it changes is deliberate.
    let (_, out) = build_theme("extra", &Demanding);
    let html = read(&out, "index.html");

    assert!(!html.contains("a key nothing declares"), "got {html}");
}

// ------------------------------------------------------------------- cards

#[test]
fn a_design_system_that_draws_cards_gets_them_written() {
    let (_, out) = build_theme("feed", &Illustrated);

    assert!(out.join("og/index.png").is_file(), "no card for the root");
    assert!(
        out.join("og/notes/note-1.png").is_file(),
        "no card for a nested page"
    );

    let html = read(&out, "index.html");
    assert!(
        html.contains("content=\"/repo/og/index.png\""),
        "got {html}"
    );
}

#[test]
fn a_card_is_registered_so_a_link_to_it_is_not_dead() {
    // The build writes it and the design system points at it; if the two
    // disagreed about the path, link checking would say so.
    let (report, _) = build_theme("feed", &Illustrated);
    assert!(report.warnings.is_empty(), "got {:?}", report.warnings);
}

#[test]
fn a_site_with_no_absolute_url_is_not_asked_for_cards() {
    // A card is only ever fetched from an absolute URL, so drawing one for a
    // site that has none would be work nobody will ever see.
    // `drafts` is the fixture with no `url`.
    let (_, out) = build_theme("drafts", &Illustrated);
    assert!(
        !out.join("og").exists(),
        "a card was drawn for a site with no url"
    );

    let html = read(&out, "index.html");
    assert!(
        !html.contains("og:image"),
        "a tag pointing at nothing: {html}"
    );
}

// ------------------------------------------------------------------- feeds

#[test]
fn a_dated_collection_gets_both_feeds() {
    let (_, out) = ok("feed");

    let atom = read(&out, "notes/atom.xml");
    let rss = read(&out, "notes/feed.xml");

    assert!(atom.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(rss.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));

    // Absolute URLs, base included, because a feed is read somewhere else.
    assert!(
        atom.contains("https://example.com/repo/notes/note-3/"),
        "got {atom}"
    );
    assert!(
        rss.contains("<link>https://example.com/repo/notes/note-3/</link>"),
        "got {rss}"
    );
}

#[test]
fn a_feed_is_newest_first_and_honours_its_limit() {
    let (_, out) = ok("feed");
    let atom = read(&out, "notes/atom.xml");

    let third = atom.find("Note 3").expect("no note 3");
    let second = atom.find("Note 2").expect("no note 2");

    assert!(third < second, "not newest first: {atom}");
    assert!(
        !atom.contains("Note 1"),
        "the limit of 2 was not applied: {atom}"
    );
}

#[test]
fn a_feed_escapes_what_a_title_can_contain() {
    let (_, out) = ok("feed");
    let atom = read(&out, "notes/atom.xml");

    assert!(atom.contains("Note 3 &amp; friends"), "got {atom}");
    assert!(
        !atom.contains("& friends"),
        "a raw ampersand reached the feed: {atom}"
    );
}

#[test]
fn the_collections_own_index_is_not_in_its_feed() {
    // It is the thing the feed is for, not an entry in it -- and it has no
    // date, which is the other reason it must not be.
    let (_, out) = ok("feed");
    assert!(!read(&out, "notes/atom.xml").contains("<title>Notes</title>\n  <entry>"));
}

#[test]
fn a_page_in_a_dated_collection_without_a_date_fails() {
    // Configuring a feed is how a site says a collection is dated.  This is
    // the build holding it to that.
    let err = fails("undated");
    assert!(matches!(err, Error::Undated { .. }));

    let message = err.render(false);
    assert!(message.contains("undated.md"), "got {message}");
    assert!(message.contains("needs a `date`"), "got {message}");
}

#[test]
fn a_feed_without_an_absolute_url_is_skipped_and_said_so() {
    // A feed is read somewhere else, so it needs absolute URLs.  Without a
    // `url` the honest thing is to write nothing and say why -- and, since no
    // feed is written, not to demand dates either.
    let (report, out) = ok("feed-no-url");

    assert!(!out.join("notes/feed.xml").exists());
    assert!(!out.join("notes/atom.xml").exists());

    assert_eq!(report.warnings.len(), 1, "got {:?}", report.warnings);
    assert!(
        report.warnings[0].contains("no url"),
        "got {:?}",
        report.warnings
    );

    let html = read(&out, "index.html");
    assert!(
        !html.contains("rel=\"alternate\""),
        "a link to a feed nobody wrote: {html}"
    );
}

// ----------------------------------------------------------- the site root

#[test]
fn a_site_root_link_lands_under_the_base() {
    let (_, out) = ok("site-root");
    let html = read(&out, "index.html");

    assert!(html.contains("href=\"/repo/docs/\""), "got {html}");
    assert!(
        !html.contains("~/docs/"),
        "a site-root link reached the output: {html}"
    );
}

#[test]
fn a_site_root_link_is_the_one_thing_the_error_page_can_use() {
    // The error page is served in place of any path, so a relative link on it
    // has no directory to resolve against and a spelled-out absolute one
    // hard-codes the base.  This is what is left.
    let (_, out) = ok("site-root");
    let html = read(&out, "404.html");

    assert!(html.contains("href=\"/repo/docs/\""), "got {html}");
}

#[test]
fn a_site_root_link_follows_an_overridden_base() {
    // The bug this fixes: the same content under a different base used to
    // produce a link to the old one, silently, because a link outside the
    // base is not checked.
    let (result, out) = build_with("site-root", Some("/elsewhere/"));
    result.expect("should build");

    let html = read(&out, "index.html");
    assert!(html.contains("href=\"/elsewhere/docs/\""), "got {html}");
    assert!(
        !html.contains("/repo/"),
        "the configured base survived: {html}"
    );
}

#[test]
fn a_tilde_in_prose_is_not_a_link() {
    let (_, out) = ok("site-root");
    let html = read(&out, "index.html");

    assert!(
        html.contains("~/Code/crabbucket"),
        "a shell path was rewritten: {html}"
    );
}

#[test]
fn a_site_root_link_to_nowhere_is_still_a_dead_link() {
    let Error::DeadLinks(dead) = fails("dead-site-root") else {
        panic!("expected dead links");
    };

    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, Reason::NoSuchRoute);
    assert_eq!(
        dead[0].target, "/gone/",
        "checked after resolution, like any other link"
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
        names(&message, "missing-title/content/index.md"),
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
    assert!(names(&err.render(false), "no-frontmatter/content/index.md"));
}

#[test]
fn an_unterminated_frontmatter_block_fails() {
    let err = fails("unterminated");
    assert!(matches!(err, Error::UnterminatedFrontmatter { .. }));
    assert!(names(&err.render(false), "unterminated/content/index.md"));
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
        "unknown-directive",
        "bad-attribute",
        "unterminated-directive",
        "dead-site-root",
        "undated",
    ] {
        let err = build(fixture).0.expect_err("should fail");
        assert!(
            err.path().is_some(),
            "`{fixture}` produced an error with no file"
        );
    }
}

// ------------------------------------------------------------------ timings

#[test]
fn a_report_says_where_the_build_spent_its_time() {
    let (report, _) = ok("ok");

    // Asserting on durations is asserting on the machine, so this asserts only
    // what cannot be false: every pass ran, so every pass took some time, and
    // the parts add up to the total.
    let timings = report.timings;
    assert!(timings.read > Duration::ZERO, "read was not measured");
    assert!(timings.render > Duration::ZERO, "render was not measured");
    assert!(timings.check > Duration::ZERO, "check was not measured");
    assert!(timings.write > Duration::ZERO, "write was not measured");

    assert_eq!(
        timings.total(),
        timings.read + timings.render + timings.check + timings.write
    );
}

#[test]
fn timings_stay_out_of_the_report_a_person_reads() {
    let (report, _) = ok("ok");

    // A build budget is the caller's business.  Nobody running `crab build`
    // asked for four durations, so `Display` does not offer them.
    assert!(!report.to_string().contains("ns"));
    assert!(!report.to_string().to_lowercase().contains("timing"));
}
