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
//! The components, exercised the way a page reaches them: as directives in
//! Markdown, through the theme's own registry.

use crabbucket::markdown;
use crabbucket::theme::Theme;
use crabbucket_ui::Standard;

/// Renders a page body with the default design system's directives.
fn render(source: &str) -> String {
    let config = crabbucket::Config::blank();
    let data = crabbucket::directive::Data::default();
    let context = crabbucket::directive::Context::new(&config, &data);

    markdown::render(source, &Standard.directives(), &context)
        .unwrap_or_else(|fault| panic!("line {}: {}", fault.line, fault.message))
        .html
}

fn fails(source: &str) -> String {
    let config = crabbucket::Config::blank();
    let data = crabbucket::directive::Data::default();
    let context = crabbucket::directive::Context::new(&config, &data);

    markdown::render(source, &Standard.directives(), &context)
        .expect_err("should have failed")
        .message
}

#[test]
fn a_callout_reaches_its_component_from_markdown() {
    let html = render(":::callout{kind = \"warn\", title = \"Careful\"}\nMind the gap.\n:::\n");

    assert!(html.contains("cb-callout cb-callout--warn"), "got {html}");
    assert!(html.contains("Careful"));
    assert!(
        html.contains("<p>Mind the gap.</p>"),
        "the body is still Markdown: {html}"
    );
}

#[test]
fn a_callout_needs_no_attributes_at_all() {
    let html = render(":::callout\nJust a note.\n:::\n");
    assert!(html.contains("cb-callout--note"), "got {html}");
}

#[test]
fn cards_nest_and_their_links_are_ordinary_links() {
    let html = render(
        ":::cards\n:::card{title = \"Routing\", href = \"routing/\"}\nHow links work.\n:::\n:::\n",
    );

    assert!(html.contains("cb-cards\">"), "got {html}");
    assert!(
        html.contains("href=\"routing/\""),
        "the link checker needs to see this: {html}"
    );
    assert!(html.contains("Routing"));
}

#[test]
fn tabs_in_one_group_share_a_radio_name_and_the_first_is_shown() {
    let html = render(
        ":::tabs{group = \"os\"}\n\
         :::tab{label = \"macOS\"}\nbrew\n:::\n\
         :::tab{label = \"Linux\"}\napt\n:::\n\
         :::\n",
    );

    let names: Vec<&str> = html
        .match_indices("name=\"")
        .map(|(at, _)| &html[at + 6..at + 18])
        .collect();
    assert_eq!(names.len(), 2, "expected two radios in {html}");
    assert_eq!(
        names[0], names[1],
        "the two tabs are in different groups: {html}"
    );

    assert_eq!(
        html.matches("checked").count(),
        1,
        "exactly one tab starts shown: {html}"
    );
    assert!(html.contains("data-tab-group=\"os\""), "got {html}");
}

#[test]
fn a_second_tab_group_on_the_page_starts_counting_again() {
    let one = ":::tabs\n:::tab{label = \"a\"}\nx\n:::\n:::tab{label = \"b\"}\ny\n:::\n:::\n";
    let html = render(&format!("{one}\n{one}"));

    assert_eq!(
        html.matches("checked").count(),
        2,
        "one shown tab per group: {html}"
    );

    let names: Vec<&str> = html
        .match_indices("name=\"")
        .map(|(at, _)| html[at + 6..].split('"').next().unwrap())
        .collect();
    assert_eq!(names.len(), 4);
    assert_eq!(names[0], names[1]);
    assert_eq!(names[2], names[3]);
    assert_ne!(
        names[0], names[2],
        "the two groups share a radio name: {html}"
    );
}

#[test]
fn steps_number_themselves() {
    let html = render(
        ":::steps\n:::step{title = \"Install\"}\nOne.\n:::\n:::step{title = \"Build\"}\nTwo.\n:::\n:::\n",
    );

    assert_eq!(html.matches("cb-steps__step").count(), 2, "got {html}");
    assert!(html.contains("Install") && html.contains("Build"));
}

#[test]
fn a_misspelled_attribute_names_the_field_and_the_component() {
    let message = fails(":::callout{knid = \"warn\"}\nx\n:::\n");
    assert!(message.contains("`callout`"), "got {message}");
    assert!(message.contains("unknown field `knid`"), "got {message}");
}

#[test]
fn a_card_without_a_href_does_not_render_a_link_to_nowhere() {
    let message = fails(":::card{title = \"Routing\"}\nx\n:::\n");
    assert!(message.contains("missing field `href`"), "got {message}");
}

#[test]
fn every_registered_directive_is_documented_by_being_usable() {
    let registered = Standard.directives();
    assert_eq!(
        registered.names(),
        ["callout", "card", "cards", "step", "steps", "tab", "tabs"]
    );
}

#[test]
fn the_router_stays_small() {
    // Around 8KB raw, which is about 3KB over the wire once a server has
    // gzipped it.  The budget is on the raw file because that is the number
    // that grows without anyone noticing, and it is deliberately close to the
    // current size: this file is meant to stay small enough to read.
    // What ships, not what is on disk: a CRLF checkout carries an extra byte
    // per line, and the build normalises them away before writing.
    let router = Standard.router_js().expect("ui has a router").replace("\r\n", "\n");
    let size = router.len();

    assert!(size < 8600, "the router has grown to {size} bytes");
}

#[test]
fn neither_client_carries_an_unresolved_placeholder() {
    let router = Standard.router_js().expect("ui has a router");
    assert!(
        !router.contains('@'),
        "a placeholder reached the router: {router}"
    );
    assert!(
        router.contains("cb-toc__link"),
        "the router does not know the contents class"
    );

    let search = Standard.search_js().expect("ui has search");
    assert!(
        !search.contains('@'),
        "a placeholder reached the search client: {search}"
    );
    assert!(
        search.contains("cb-search__result"),
        "the client does not know the result class"
    );
}

#[test]
fn the_search_client_is_syntactically_valid_javascript() {
    use std::io::Write;
    use std::process::Command;

    let Ok(probe) = Command::new("node").arg("--version").output() else {
        eprintln!("skipping: node is not installed");
        return;
    };

    if !probe.status.success() {
        eprintln!("skipping: node is not usable");
        return;
    }

    let path = std::env::temp_dir().join("crabbucket-search-check.js");
    let mut file = std::fs::File::create(&path).expect("cannot write the client out");
    file.write_all(Standard.search_js().expect("ui has search").as_bytes())
        .expect("cannot write the client out");
    drop(file);

    let checked = Command::new("node")
        .arg("--check")
        .arg(&path)
        .output()
        .expect("node failed");
    let _ = std::fs::remove_file(&path);

    assert!(
        checked.status.success(),
        "the search client is not valid JavaScript:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
}

#[test]
fn the_router_lets_a_design_system_hook_a_navigation() {
    // Without this a theme with anything interactive has to fork the router,
    // which silently costs it prefetching, scroll-spy, tab memory and copy
    // buttons.  One event fixes it for everyone.
    let js = Standard.router_js().expect("ui has a router");

    assert!(
        js.contains("crabbucket:render"),
        "no extension point in the router"
    );
    assert!(js.contains("CustomEvent"), "the hook is not an event: {js}");

    // The built-in enhancers go through the same door a theme's would, so the
    // door cannot rot: if it breaks, the default design system breaks first.
    for enhancer in ["tabs", "spy", "copy"] {
        assert!(
            js.contains(&format!("addEventListener(RENDERED, {enhancer})")),
            "`{enhancer}` is still called by name rather than listening"
        );
    }
}

#[test]
fn the_router_moves_focus_and_announces_itself() {
    let js = Standard.router_js().expect("ui has a router");

    for behaviour in [
        "aria-live",
        "focus(",
        "scrollRestoration",
        "prefers-reduced-motion",
    ] {
        assert!(
            js.contains(behaviour),
            "the router no longer does `{behaviour}`"
        );
    }
}

// ------------------------------------------------------------------ contents

use crabbucket::Heading;
use crabbucket_ui::contents;

fn heading(level: u8, id: &str) -> Heading {
    Heading {
        id: id.to_string(),
        level,
        text: id.to_uppercase(),
    }
}

#[test]
fn a_short_page_gets_no_table_of_contents() {
    // Two entries take a column of the page to tell the reader something the
    // page already told them.
    let short = [heading(2, "one"), heading(2, "two")];
    assert!(contents(&short).is_none());

    let long = [heading(2, "one"), heading(2, "two"), heading(2, "three")];
    assert!(contents(&long).is_some());
}

#[test]
fn only_h2_and_h3_are_listed() {
    let headings = [
        heading(1, "title"),
        heading(2, "one"),
        heading(3, "one-a"),
        heading(4, "detail"),
        heading(2, "two"),
    ];

    let html = contents(&headings)
        .expect("three qualifying headings")
        .beside
        .into_string();

    assert!(html.contains("#one") && html.contains("#one-a") && html.contains("#two"));
    assert!(
        !html.contains("#title"),
        "the page title is not a contents entry: {html}"
    );
    assert!(
        !html.contains("#detail"),
        "h4 is detail, not contents: {html}"
    );
}

#[test]
fn h3s_nest_under_the_h2_they_follow() {
    let headings = [
        heading(2, "one"),
        heading(3, "one-a"),
        heading(3, "one-b"),
        heading(2, "two"),
    ];

    let html = contents(&headings)
        .expect("four headings")
        .beside
        .into_string();

    // The nested list opens after `one` and closes before `two`.
    let one = html.find("#one\"").expect("no #one");
    let nested = html.find("<ul").expect("no list at all");
    let inner = html[one..]
        .find("<ul")
        .map(|at| at + one)
        .expect("no nested list");
    let two = html.find("#two\"").expect("no #two");

    assert!(nested < one, "the outer list should come first");
    assert!(inner < two, "the h3s did not nest: {html}");
    assert_eq!(
        html.matches("<ul").count(),
        2,
        "expected exactly one nested list: {html}"
    );
}

#[test]
fn the_contents_is_rendered_twice_and_css_chooses() {
    let headings = [heading(2, "one"), heading(2, "two"), heading(2, "three")];
    let both = contents(&headings).expect("three headings");

    assert!(
        both.beside
            .into_string()
            .contains("aria-label=\"On this page\"")
    );

    let folded = both.folded.into_string();
    assert!(folded.starts_with("<details"), "got {folded}");
    assert!(folded.contains("cb-toc cb-toc--folded"), "got {folded}");
}

#[test]
fn every_contents_link_is_a_fragment_the_page_actually_has() {
    // The ids come from the same list the link checker validates fragments
    // against, so a contents entry cannot point at a heading that is not there.
    let headings = [heading(2, "one"), heading(2, "two"), heading(3, "two-a")];
    let html = contents(&headings)
        .expect("three headings")
        .beside
        .into_string();

    for heading in &headings {
        assert!(
            html.contains(&format!("href=\"#{}\"", heading.id)),
            "missing {}",
            heading.id
        );
    }
}

#[test]
fn the_router_is_syntactically_valid_javascript() {
    use std::io::Write;
    use std::process::Command;

    // Everything else about this file is checked by reading it, which is not a
    // check.  `node --check` is, where node exists.
    let Ok(probe) = Command::new("node").arg("--version").output() else {
        eprintln!("skipping: node is not installed");
        return;
    };

    if !probe.status.success() {
        eprintln!("skipping: node is not usable");
        return;
    }

    let path = std::env::temp_dir().join("crabbucket-router-check.js");
    let mut file = std::fs::File::create(&path).expect("cannot write the router out");
    file.write_all(Standard.router_js().expect("ui has a router").as_bytes())
        .expect("cannot write the router out");
    drop(file);

    let checked = Command::new("node")
        .arg("--check")
        .arg(&path)
        .output()
        .expect("node failed");
    let _ = std::fs::remove_file(&path);

    assert!(
        checked.status.success(),
        "the router is not valid JavaScript:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
}
