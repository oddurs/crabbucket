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

//! Ferrite: the design system the book builds, as code the compiler sees.
//!
//! Chapters 4, 5 and 6 of the book walk from an empty crate to a house style
//! that a fleet of sites depends on.  Everything they show in Rust is in this
//! crate, and a test asserts that every `rust` block in those chapters appears
//! here verbatim.
//!
//! That inversion is the point.  A book with a snippet that no longer compiles
//! is worse than no book, because it is confidently wrong -- so the snippets
//! are not listings that a checker inspects, they are this crate's source, and
//! the check that they compile is `cargo build`.
//!
//! It is also, incidentally, a third design system.  The second one caught
//! three bugs the first could not; there was no reason to expect the third to
//! be free, and it was not.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crabbucket::directive::Directives;
use crabbucket::style::{Style, StyleSheet};
use crabbucket::theme::{NavItem, Page, Theme};
use crabbucket::{Config, PageMeta, Url};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde::Deserialize;

pub mod fleet;

/// The namespace every class in this design system carries.
pub const NS: &str = "fe";

/// Design tokens, generated from this crate's own `design/tokens.toml`.
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

/// The layouts Ferrite offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Prose in one column.
    #[default]
    Page,
    /// Prose with the rest of its section listed beside it.
    #[serde(alias = "landing")]
    Docs,
}

/// The frontmatter Ferrite reads beyond what every page has.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Extra {
    /// A sentence that stands above the prose.
    #[serde(default)]
    pub summary: Option<String>,
}

/// The Ferrite design system.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ferrite;

impl Theme for Ferrite {
    type Layout = Layout;
    type Extra = Extra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        let body = match page.meta.layout {
            Layout::Page => prose(page.html),
            Layout::Docs => {
                let section = page.route.split('/').next().unwrap_or("");
                docs(&page.section(section), prose(page.html))
            }
        };

        document(page.config, page.meta, &page.nav(), body).into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([PAGE, PROSE, NOTE]);
        format!("{}\n{}", tok::CSS, sheet.render())
    }

    fn directives(&self) -> Directives {
        let mut directives = Directives::new();

        directives.add("callout", |props: Callout, body| {
            html! {
                aside class=(NOTE.class()) {
                    @if let Some(title) = &props.title {
                        strong class=(NOTE.element("title")) { (title) }
                    }
                    (body)
                }
            }
        });

        directives
    }
}

/// The page shell.
pub const PAGE: Style = Style::new(NS, "page", include_str!("styles/page.css"));

/// Everything Markdown produces.
pub const PROSE: Style = Style::new(NS, "prose", include_str!("styles/prose.css"));

/// An aside beside the prose.
pub const NOTE: Style = Style::new(NS, "note", include_str!("styles/note.css"));

/// `:::callout{title = "Careful"}`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Callout {
    /// A heading for the aside.
    #[serde(default)]
    pub title: Option<String>,
}

/// Renders a body of Markdown-derived HTML.
///
/// The HTML comes from the site's own content, which is trusted, so it is
/// emitted unescaped.
pub fn prose(html_fragment: &str) -> Markup {
    html! { div class=(PROSE.class()) { (PreEscaped(html_fragment)) } }
}

/// Prose with the rest of its section listed beside it.
pub fn docs(section: &[NavItem], content: Markup) -> Markup {
    html! {
        nav { (links(section)) }
        (content)
    }
}

/// A list of links, with the current one marked.
pub fn links(items: &[NavItem]) -> Markup {
    html! {
        @for item in items {
            a href=(item.href) aria-current=[item.current.then_some("page")] { (item.label) }
        }
    }
}

/// The whole document.
pub fn document(
    config: &Config,
    meta: &PageMeta<Layout, Extra>,
    nav: &[NavItem],
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
            }
            body class=(PAGE.class()) {
                header class=(PAGE.element("masthead")) { (links(nav)) }
                main class=(PAGE.element("main")) {
                    @if let Some(summary) = &meta.extra.summary {
                        p { em { (summary) } }
                    }
                    (content)
                }
            }
        }
    }
}
