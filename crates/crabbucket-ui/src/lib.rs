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
//! The default design system and component kit for crabbucket.
//!
//! Components are ordinary functions returning [`maud::Markup`], so their
//! props are typed, their HTML is checked when the crate compiles, and there
//! is no template language and no runtime between the two.  Every colour and
//! length they use comes from [`tok`], which is generated from
//! `design/tokens.toml` by this crate's build script.
//!
//! Every component's styles are declared beside it as a [`Style`], namespaced
//! `cb`, and the theme composes the stylesheet from them.  A component in
//! another design system may be called `card` too without either one winning.
//!
//! Swapping this crate for another one is how a site changes design system.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crabbucket::style::{Style, StyleSheet};
use crabbucket::theme::{NavItem, Page, Theme};
use crabbucket::{Config, PageMeta, Url};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde::Deserialize;

/// The namespace every class in this design system carries.
pub const NS: &str = "cb";

/// Design tokens, generated from `design/tokens.toml`.
///
/// Each constant is a `var(--group-name, fallback)` reference, so a component
/// picks up a theme override at runtime while still failing to compile if the
/// token itself is renamed.
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

/// The layouts this design system offers.
///
/// `layout = "docs"` in a page's frontmatter deserializes into this enum, so a
/// layout that does not exist fails the build naming the file, the line, and
/// the layouts that do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Prose, centred, no sidebar.  The default.
    #[default]
    Page,
    /// Prose with a sidebar listing the rest of the section.
    Docs,
    /// Wider measure and a larger opening, for a front page.
    Landing,
}

impl Layout {
    /// The modifier suffix this layout puts on the page shell.
    pub fn slug(self) -> &'static str {
        match self {
            Layout::Page => "page",
            Layout::Docs => "docs",
            Layout::Landing => "landing",
        }
    }
}

/// The default design system.
///
/// A site swaps design system by handing [`crabbucket::build`] a different
/// implementation of [`Theme`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Standard;

impl Theme for Standard {
    type Layout = Layout;

    fn render(&self, page: &Page<'_, Layout>) -> String {
        let nav = page.nav();

        let content = match page.meta.layout {
            Layout::Page | Layout::Landing => prose(page.html),
            Layout::Docs => {
                let section = page.route.split('/').next().unwrap_or("");
                docs(&page.section(section), prose(page.html))
            }
        };

        document(page.config, page.meta, &nav, content).into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([SITE, MASTHEAD, PROSE, DOCS, CALLOUT, COLOPHON]);
        format!("{}\n{}", tok::CSS, sheet.render())
    }

    fn router_js(&self) -> String {
        router_js()
    }
}

/// The kind of a callout, which decides its accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Neutral information.
    Note,
    /// Something the reader can get wrong.
    Warn,
}

impl Kind {
    /// The modifier suffix used in the class name.
    pub fn slug(self) -> &'static str {
        match self {
            Kind::Note => "note",
            Kind::Warn => "warn",
        }
    }
}

/// An aside that stands apart from the prose around it.
pub fn callout(kind: Kind, body: Markup) -> Markup {
    html! {
        aside class=(CALLOUT.with(kind.slug())) {
            div class=(CALLOUT.element("body")) { (body) }
        }
    }
}

/// A list of links, with the current one marked.
pub fn nav_list(items: &[NavItem]) -> Markup {
    html! {
        @for item in items {
            a href=(item.href) aria-current=[item.current.then_some("page")] { (item.label) }
        }
    }
}

/// Prose with a section sidebar beside it.
pub fn docs(section: &[NavItem], content: Markup) -> Markup {
    html! {
        div class=(DOCS.class()) {
            nav class=(DOCS.element("side")) { (nav_list(section)) }
            div class=(DOCS.element("main")) { (content) }
        }
    }
}

/// Renders a body of Markdown-derived HTML inside the prose wrapper.
///
/// The HTML comes from the site's own content, which is trusted, so it is
/// emitted unescaped.
pub fn prose(html_fragment: &str) -> Markup {
    html! { div class=(PROSE.class()) { (PreEscaped(html_fragment)) } }
}

/// The whole document: head, navigation, content, footer.
pub fn document(
    config: &Config,
    meta: &PageMeta<Layout>,
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
                @if config.router {
                    script defer src=(Url::asset(config, "router.js")) {}
                }
            }
            body class=(SITE.with(meta.layout.slug())) {
                header class=(MASTHEAD.class()) {
                    a class=(MASTHEAD.element("home")) href=(Url::new(config, "")) {
                        (config.title)
                    }
                    nav class=(MASTHEAD.element("nav")) { (nav_list(nav)) }
                }
                main class=(SITE.element("main")) { (content) }
                footer class=(COLOPHON.class()) {
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

/// The site's complete stylesheet: the generated token block, then every
/// component's styles.
pub fn stylesheet() -> String {
    Standard.stylesheet()
}

/// The client-side router, as described in section 7 of `doc/DESIGN`.
///
/// It intercepts same-origin, same-document link clicks, fetches the target,
/// swaps `<main>` and the title, and drives the View Transitions API when the
/// browser has one.  With JavaScript off, or on a browser that fails any of
/// its guards, navigation is what it always was.
pub fn router_js() -> String {
    ROUTER.replace("@NAV@", &format!(".{}", MASTHEAD.element("nav")))
}

/// The page shell: reset, typography defaults, and the main column.
pub const SITE: Style = Style::new(NS, "site", include_str!("styles/site.css"));

/// The masthead across the top of every page.
pub const MASTHEAD: Style = Style::new(NS, "masthead", include_str!("styles/masthead.css"));

/// Prose: everything Markdown produces, including the framework's own
/// `heading-anchor`, `code` and `tok-` classes.
pub const PROSE: Style = Style::new(NS, "prose", include_str!("styles/prose.css"));

/// The documentation layout: a sidebar beside the content.
pub const DOCS: Style = Style::new(NS, "docs", include_str!("styles/docs.css"));

/// The callout component.
pub const CALLOUT: Style = Style::new(NS, "callout", include_str!("styles/callout.css"));

/// The footer.
pub const COLOPHON: Style = Style::new(NS, "colophon", include_str!("styles/colophon.css"));

const ROUTER: &str = include_str!("router.js");

#[cfg(test)]
mod tests {
    use crabbucket::style::StyleSheet;

    use super::{CALLOUT, DOCS, Kind, MASTHEAD, NS, PROSE, SITE, callout};

    #[test]
    fn every_class_in_the_stylesheet_carries_the_namespace() {
        let mut sheet = StyleSheet::new();
        sheet.extend([SITE, MASTHEAD, PROSE, DOCS, CALLOUT]);
        let css = sheet.render();

        assert!(!css.contains('&'), "an unresolved ampersand escaped");

        for line in css.lines() {
            let Some(class) = line.trim().strip_prefix('.') else {
                continue;
            };
            let class = class
                .split([' ', ',', ':', '{', '>', '[', '.'])
                .next()
                .unwrap_or("");
            let framework =
                class.starts_with("tok-") || crabbucket::style::FRAMEWORK_CLASSES.contains(&class);
            assert!(
                framework || class.starts_with(&format!("{NS}-")),
                "`.{class}` is neither namespaced nor a framework class"
            );
        }
    }

    #[test]
    fn markup_and_styles_agree_on_the_class_names() {
        let markup = callout(Kind::Warn, maud::html! { p { "hi" } }).into_string();
        assert!(markup.contains("cb-callout cb-callout--warn"));
        assert!(markup.contains("cb-callout__body"));

        let css = CALLOUT.render();
        assert!(css.contains(".cb-callout--warn"));
        assert!(css.contains(".cb-callout__body"));
    }
}
