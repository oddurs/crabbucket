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
    markdown::render(source, &Standard.directives())
        .unwrap_or_else(|fault| panic!("line {}: {}", fault.line, fault.message))
        .html
}

fn fails(source: &str) -> String {
    markdown::render(source, &Standard.directives())
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
    // 3.7KB raw is about 1.6KB over the wire once a server has gzipped it.
    // The budget is on the raw file because that is the number that grows
    // without anyone noticing.
    let size = Standard.router_js().len();
    assert!(size < 4096, "the router has grown to {size} bytes");
}

#[test]
fn the_router_moves_focus_and_announces_itself() {
    let js = Standard.router_js();

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
