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
//! A second design system for crabbucket, deliberately unlike the first.
//!
//! `crabbucket-ui` is dark, has three layouts, ships a router and a search
//! client, and registers seven directives.  Plain is light, has one layout,
//! ships no JavaScript at all, and registers nothing.  It sets a document in
//! one serif column and stops.
//!
//! It exists because a trait with one implementation is a guess about what the
//! abstraction needs.  Writing it changed three things in crabbucket, and each
//! of those changes is the deliverable rather than the theme itself:
//!
//! 1. `Theme::router_js` and `Theme::search_js` return `Option<String>`.  They
//!    returned `String` and `&str`, so a design system without a router had no
//!    way to say so and a site asking for one got an empty file.  A site that
//!    asks now gets a warning and a page that works.
//! 2. The token generator moved into `crabbucket-tokens`.  It lived in
//!    `crabbucket-ui`'s build script, so the second theme's first draft was a
//!    forty-line copy of it.
//! 3. That generator learned which scheme a palette *is*.  It assumed a dark
//!    base with light overrides, because that was the only palette it had ever
//!    seen.  Plain is light-first.
//!
//! It also settled a question the first design system could not ask: what a
//! page's `layout` and its `:::directives` are portable across.  They name
//! things in a design system, so a theme meant to be swapped in has to answer
//! to the names the content already uses.  Plain does, in one line of
//! `serde(alias)` and one module of deliberately plain components -- and
//! renders crabbucket's own documentation site unmodified.
//!
//! Nothing else in `crabbucket` needed changing to make Plain render.  That
//! part went right.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crabbucket::directive::Directives;
use crabbucket::style::{Style, StyleSheet};
use crabbucket::theme::{FeedLink, NavItem, Page, Theme};
use crabbucket::{Config, PageMeta, Url};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde::Deserialize;

/// The namespace every class in this design system carries.
pub const NS: &str = "pl";

/// Design tokens, generated from this crate's own `design/tokens.toml`.
///
/// A design system owns its palette.  Reaching into crabbucket's would make
/// every theme a fork of the default one.
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

pub mod components;

/// The layouts this design system offers.
///
/// There is one, and the aliases are how it says so.  A site written for a
/// design system with three layouts keeps building here, because "docs" and
/// "landing" are names for a distinction Plain does not make -- while a layout
/// nobody has heard of still fails the build.
///
/// This is the portability contract, and it is worth stating plainly: a page's
/// `layout` names something in its design system, so a theme meant to be
/// swapped in has to answer to the names the content already uses.  One line
/// of `serde(alias)` is the whole cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// A document in one column.
    #[default]
    #[serde(alias = "docs", alias = "landing")]
    Page,
}

/// The frontmatter Plain reads beyond what every page has.
///
/// A `summary` is a sentence that stands above the prose and says what the
/// page is for.  Declaring it here is what makes it exist: a design system
/// that says nothing gets nothing, and a site cannot add a field its design
/// system does not read.
///
/// It is optional, so a page without one still builds.  Making it required is
/// one word, and then a page without one fails naming itself.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Extra {
    /// A sentence above the prose.
    #[serde(default)]
    pub summary: Option<String>,
}

/// The plain design system.
#[derive(Debug, Clone, Copy, Default)]
pub struct Plain;

impl Theme for Plain {
    type Layout = Layout;
    type Extra = Extra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        document(
            page.config,
            page.meta,
            &page.nav(),
            page.feeds,
            prose(page.html),
        )
        .into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([PAGE, PROSE]);
        sheet.extend(components::STYLES.iter().copied());
        format!("{}\n{}", tok::CSS, sheet.render())
    }

    fn directives(&self) -> Directives {
        components::directives()
    }

    // No router and no search.  The defaults say so, and a site that asks for
    // one anyway is told rather than handed an empty file.
}

/// The page shell.
pub const PAGE: Style = Style::new(NS, "page", include_str!("styles/page.css"));

/// Everything Markdown produces.
pub const PROSE: Style = Style::new(NS, "prose", include_str!("styles/prose.css"));

/// A list of links, with the current one marked.
pub fn nav_list(items: &[NavItem]) -> Markup {
    html! {
        @for item in items {
            a href=(item.href) aria-current=[item.current.then_some("page")] { (item.label) }
        }
    }
}

/// Renders a body of Markdown-derived HTML.
///
/// The HTML comes from the site's own content, which is trusted, so it is
/// emitted unescaped.
pub fn prose(html_fragment: &str) -> Markup {
    html! { div class=(PROSE.class()) { (PreEscaped(html_fragment)) } }
}

/// The whole document.
///
/// No `<script>`, under any configuration: this design system has nothing to
/// put in one, so it never writes a tag pointing at a file that will not
/// exist.
pub fn document(
    config: &Config,
    meta: &PageMeta<Layout, Extra>,
    nav: &[NavItem],
    feeds: &[FeedLink],
    content: Markup,
) -> Markup {
    let description = meta.description.as_deref().unwrap_or(&config.description);

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (meta.title) " — " (config.title) }
                @if !description.is_empty() {
                    meta name="description" content=(description);
                }
                link rel="stylesheet" href=(Url::asset(config, "site.css"));
                @for feed in feeds {
                    link rel="alternate"
                         type="application/atom+xml"
                         title=(feed.title)
                         href=(feed.atom);
                    link rel="alternate"
                         type="application/rss+xml"
                         title=(feed.title)
                         href=(feed.rss);
                }
            }
            body class=(PAGE.class()) {
                header class=(PAGE.element("masthead")) {
                    a class=(PAGE.element("home")) href=(Url::new(config, "")) { (config.title) }
                    nav class=(PAGE.element("nav")) { (nav_list(nav)) }
                }
                main class=(PAGE.element("main")) { (content) }
                footer class=(PAGE.element("colophon")) {
                    p {
                        "Built with "
                        a href="https://github.com/oddurs/crabbucket" { "crabbucket" }
                        "."
                    }
                }
            }
        }
    }
}
