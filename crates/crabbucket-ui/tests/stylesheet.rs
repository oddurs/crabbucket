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
//! Every class the markup emits must have a rule, and every rule must be
//! namespaced.
//!
//! This test exists because splitting one stylesheet into six silently lost
//! the landing-layout rules, and nothing noticed until the markup and the CSS
//! were compared directly.  A design system whose classes and rules can drift
//! apart is a design system that will.

use std::collections::BTreeSet;

use crabbucket::markdown::Heading;
use crabbucket::theme::{Page, PageMeta, PageRef, SiteIndex, Theme};
use crabbucket::{Config, style};
use crabbucket_ui::{Layout, NS, Standard};

/// Modifiers that exist as hooks for a site to hang its own rules on, and so
/// are deliberately unstyled here.
const HOOKS: &[&str] = &["cb-site--page", "cb-site--docs", "cb-docs__main"];

fn config() -> Config {
    Config {
        title: "Fixture".into(),
        description: "A fixture site.".into(),
        url: None,
        base: "/repo/".into(),
        router: false,
    }
}

fn meta(layout: Layout) -> PageMeta<Layout> {
    PageMeta {
        title: "Page".into(),
        description: None,
        layout,
        nav_order: Some(1),
        order: None,
        nav_label: None,
        draft: false,
    }
}

fn index() -> SiteIndex {
    SiteIndex::new(vec![
        PageRef {
            route: String::new(),
            label: "Home".into(),
            nav_order: Some(1),
            order: None,
        },
        PageRef {
            route: "docs".into(),
            label: "Docs".into(),
            nav_order: Some(2),
            order: None,
        },
        PageRef {
            route: "docs/one".into(),
            label: "One".into(),
            nav_order: None,
            order: None,
        },
    ])
}

/// Renders one page in each layout and collects every class in the markup.
fn classes_in_markup() -> BTreeSet<String> {
    let config = config();
    let site = index();
    let body = "<h2 id=\"a\">A</h2><pre class=\"code\"><code>x</code></pre>";

    // Enough headings that the table of contents renders; below the minimum
    // it is skipped, and its classes would go untested.
    let headings: Vec<Heading> = ["a", "b", "c"]
        .iter()
        .enumerate()
        .map(|(index, id)| Heading {
            id: (*id).to_string(),
            level: if index == 2 { 3 } else { 2 },
            text: id.to_uppercase(),
        })
        .collect();

    let mut found = BTreeSet::new();

    for layout in [Layout::Page, Layout::Docs, Layout::Landing] {
        let meta = meta(layout);
        let page = Page {
            config: &config,
            meta: &meta,
            route: "docs/one",
            html: body,
            headings: &headings,
            site: &site,
        };

        let html = Standard.render(&page);
        let mut rest = html.as_str();

        while let Some(start) = rest.find("class=\"") {
            rest = &rest[start + 7..];
            let Some(end) = rest.find('"') else { break };
            found.extend(rest[..end].split_whitespace().map(str::to_string));
            rest = &rest[end + 1..];
        }
    }

    found
}

#[test]
fn every_class_in_the_markup_has_a_rule() {
    let css = Standard.stylesheet();

    let missing: Vec<String> = classes_in_markup()
        .into_iter()
        .filter(|class| class.starts_with(NS))
        .filter(|class| !HOOKS.contains(&class.as_str()))
        .filter(|class| !css.contains(&format!(".{class}")))
        .collect();

    assert!(missing.is_empty(), "classes with no rule: {missing:?}");
}

#[test]
fn every_class_in_the_markup_is_namespaced_or_belongs_to_the_framework() {
    for class in classes_in_markup() {
        let framework = class.starts_with(style::HIGHLIGHT_PREFIX)
            || style::FRAMEWORK_CLASSES.contains(&class.as_str());

        assert!(
            framework || class.starts_with(&format!("{NS}-")),
            "`{class}` is neither namespaced nor a framework class"
        );
    }
}

#[test]
fn the_stylesheet_carries_the_tokens_it_uses() {
    let css = Standard.stylesheet();

    for token in [
        "--color-surface",
        "--space-md",
        "--measure-prose",
        "--syntax-keyword",
    ] {
        assert!(css.contains(&format!("{token}:")), "{token} is not defined");
    }

    assert!(
        !css.contains('&'),
        "an unresolved ampersand reached the stylesheet"
    );
}

#[test]
fn the_router_carries_the_themes_own_class_names() {
    let js = Standard.router_js();
    assert!(
        js.contains("cb-masthead__nav"),
        "the router does not know the theme's nav class"
    );
    assert!(
        !js.contains("@NAV@"),
        "an unresolved placeholder reached the router"
    );
}
