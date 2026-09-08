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

use crabbucket::directive::Directives;
use crabbucket::style::{Style, StyleSheet};
use crabbucket::theme::{FeedLink, NavItem, NoExtra, Page, Theme};
use crabbucket::{Config, Heading, PageMeta, Url};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde::Deserialize;

/// The namespace every class in this design system carries.
pub const NS: &str = "cb";

pub mod components;

pub use components::{Kind, callout};

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
    type Extra = NoExtra;

    fn render(&self, page: &Page<'_, Self>) -> String {
        let nav = page.nav();

        let content = match page.meta.layout {
            Layout::Page | Layout::Landing => prose(page.html),
            Layout::Docs => {
                let section = page.route.split('/').next().unwrap_or("");
                let (previous, next) = page.neighbours(section);

                docs(
                    &page.section(section),
                    contents(page.headings),
                    html! {
                        (prose(page.html))
                        (neighbours(previous.as_ref(), next.as_ref()))
                    },
                )
            }
        };

        document(page.config, page.meta, &nav, page.feeds, page.card, content).into_string()
    }

    fn stylesheet(&self) -> String {
        let mut sheet = StyleSheet::new();
        sheet.extend([
            SITE, MASTHEAD, PROSE, DOCS, TOC, NEIGHBOURS, COPY, SEARCH, COLOPHON,
        ]);
        sheet.extend(components::STYLES.iter().copied());
        format!("{}\n{}", tok::CSS, sheet.render())
    }

    fn directives(&self) -> Directives {
        components::directives()
    }

    fn og_image(&self, page: &Page<'_, Self>) -> Option<Vec<u8>> {
        // The card is drawn from this design system's own tokens, so it looks
        // like the site it belongs to without anybody restating the palette.
        let card = crabbucket_og::Card {
            title: &page.meta.title,
            site: &page.config.title,
            palette: crabbucket_og::Palette {
                background: hex(tok::color::SURFACE),
                foreground: hex(tok::color::TEXT),
                muted: hex(tok::color::TEXT_MUTED),
                accent: hex(tok::color::ACCENT),
            },
        };

        crabbucket_og::cached(&page.cache.join("og"), &card).ok()
    }

    fn search_js(&self) -> Option<String> {
        Some(search_js())
    }

    fn router_js(&self) -> Option<String> {
        Some(router_js())
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

/// Prose with a section sidebar beside it, and a table of contents if the
/// page is long enough to want one.
pub fn docs(section: &[NavItem], contents: Option<Contents>, content: Markup) -> Markup {
    html! {
        div class=(DOCS.class()) {
            nav class=(DOCS.element("side")) { (nav_list(section)) }
            div class=(DOCS.element("main")) {
                @if let Some(contents) = &contents { (contents.folded) }
                (content)
            }
            @if let Some(contents) = &contents {
                div class=(DOCS.element("aside")) { (contents.beside) }
            }
        }
    }
}

/// Links to the pages either side of this one.
///
/// Each is labelled with the page's title rather than with "Previous" and
/// "Next" alone, because the title is the useful half and the direction is
/// already obvious from which side of the page it is on.  An end of the
/// section omits its side rather than rendering a disabled link.
pub fn neighbours(previous: Option<&NavItem>, next: Option<&NavItem>) -> Markup {
    if previous.is_none() && next.is_none() {
        return html! {};
    }

    html! {
        nav class=(NEIGHBOURS.class()) aria-label="Section" {
            @if let Some(item) = previous {
                a class=(NEIGHBOURS.element("link")) href=(item.href) rel="prev" {
                    span class=(NEIGHBOURS.element("direction")) { "Previous" }
                    span class=(NEIGHBOURS.element("title")) { (item.label) }
                }
            }
            @if let Some(item) = next {
                a class=(format!("{0} {0}--next", NEIGHBOURS.element("link")))
                  href=(item.href) rel="next" {
                    span class=(NEIGHBOURS.element("direction")) { "Next" }
                    span class=(NEIGHBOURS.element("title")) { (item.label) }
                }
            }
        }
    }
}

/// A table of contents, in both of its renderings.
///
/// The same list appears twice in the document, and CSS decides which is
/// shown: beside the prose where there is a column for it, folded into a
/// disclosure above the prose where there is not.  Rendering it twice costs a
/// few hundred bytes and buys a layout that needs neither script nor knowledge
/// of the viewport at build time.
pub struct Contents {
    /// The sidebar rendering.
    pub beside: Markup,
    /// The disclosure rendering.
    pub folded: Markup,
}

/// The minimum number of headings before a table of contents earns its place.
///
/// A contents list with two entries is noise: it takes a column of the page to
/// tell the reader something the page already told them.
const CONTENTS_MINIMUM: usize = 3;

/// A table of contents, or nothing.
///
/// Only `h2` and `h3` appear.  `h1` is the page title, which the reader is
/// looking at, and anything below `h3` is detail a contents list should not be
/// competing with.
pub fn contents(headings: &[Heading]) -> Option<Contents> {
    let listed: Vec<&Heading> = headings
        .iter()
        .filter(|heading| matches!(heading.level, 2 | 3))
        .collect();

    if listed.len() < CONTENTS_MINIMUM {
        return None;
    }

    Some(Contents {
        beside: html! {
            nav class=(TOC.class()) aria-label="On this page" {
                span class=(TOC.element("label")) { "On this page" }
                (contents_list(&listed))
            }
        },
        folded: html! {
            details class=(TOC.with("folded")) {
                summary { "On this page" }
                (contents_list(&listed))
            }
        },
    })
}

/// The nested list itself, shared by both renderings.
///
/// The loop is plain Rust rather than a template construct: nesting `h3`s
/// under their `h2` needs to skip forward by however many children it just
/// consumed, and expressing that inside a markup macro reads far worse than
/// building the items first.
fn contents_list(listed: &[&Heading]) -> Markup {
    let mut items: Vec<Markup> = Vec::new();
    let mut index = 0;

    while index < listed.len() {
        let heading = listed[index];
        let children: &[&Heading] = if heading.level == 2 {
            children_of(listed, index)
        } else {
            &[]
        };

        items.push(html! {
            li {
                a class=(TOC.element("link")) href=(format!("#{}", heading.id)) {
                    (heading.text)
                }
                @if !children.is_empty() { (contents_list(children)) }
            }
        });

        index += 1 + children.len();
    }

    html! {
        ul class=(TOC.element("list")) {
            @for item in &items { (item) }
        }
    }
}

/// The `h3`s that follow an `h2`, up to the next `h2`.
fn children_of<'a>(listed: &'a [&'a Heading], at: usize) -> &'a [&'a Heading] {
    let rest = &listed[at + 1..];
    let end = rest
        .iter()
        .position(|heading| heading.level == 2)
        .unwrap_or(rest.len());
    &rest[..end]
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
    meta: &PageMeta<Layout, NoExtra>,
    nav: &[NavItem],
    feeds: &[FeedLink],
    card: Option<&Url>,
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
                @if let Some(card) = card {
                    meta property="og:image" content=(card);
                    meta property="og:image:width" content=(crabbucket_og::WIDTH);
                    meta property="og:image:height" content=(crabbucket_og::HEIGHT);
                    meta name="twitter:card" content="summary_large_image";
                }
                meta property="og:title" content=(meta.title);
                meta property="og:type" content="website";
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
                @if config.router {
                    script defer src=(Url::asset(config, "router.js")) {}
                }
                @if config.search {
                    script defer src=(Url::asset(config, "search.js")) {}
                }
            }
            body class=(SITE.with(meta.layout.slug())) {
                header class=(MASTHEAD.class()) {
                    a class=(MASTHEAD.element("home")) href=(Url::new(config, "")) {
                        (config.title)
                    }
                    nav class=(MASTHEAD.element("nav")) { (nav_list(nav)) }
                    @if config.search { (search_form(config)) }
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

/// The literal value out of a `var(--name, #value)` token.
///
/// A token is a CSS reference so that a runtime theme override reaches it.  An
/// image is drawn before any browser exists, so it needs the value the
/// reference falls back to.
fn hex(token: &str) -> &str {
    token
        .rsplit_once(", ")
        .and_then(|(_, fallback)| fallback.strip_suffix(')'))
        .unwrap_or(token)
}

/// The search box.
///
/// It ships `hidden` and the client unhides it, so a reader without script
/// sees no search box rather than one that does nothing.  The index URL is
/// built through [`Url::asset`] like every other, so search works under a base
/// path without knowing there is one.
pub fn search_form(config: &Config) -> Markup {
    html! {
        form class=(SEARCH.class())
             role="search"
             data-index=(Url::asset(config, "search.json"))
             hidden {
            input class=(SEARCH.element("input"))
                  type="search"
                  placeholder="Search…"
                  aria-label="Search this site"
                  autocomplete="off";
            div class=(SEARCH.element("results")) hidden {}
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
    ROUTER
        .replace("@NAV@", &format!(".{}", MASTHEAD.element("nav")))
        .replace("@TOC@", &format!(".{}", TOC.element("link")))
        .replace("@COPY@", &COPY.class())
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

/// The table of contents.
pub const TOC: Style = Style::new(NS, "toc", include_str!("styles/toc.css"));

/// The search box, and the results under it.
pub const SEARCH: Style = Style::new(NS, "search", include_str!("styles/search.css"));

/// The copy button the router adds to code blocks.
pub const COPY: Style = Style::new(NS, "copy", include_str!("styles/copy.css"));

/// The links either side of a page within its section.
pub const NEIGHBOURS: Style = Style::new(NS, "neighbours", include_str!("styles/neighbours.css"));

/// The footer.
pub const COLOPHON: Style = Style::new(NS, "colophon", include_str!("styles/colophon.css"));

/// The search client, with this design system's own class names in it.
pub fn search_js() -> String {
    SEARCH_CLIENT
        .replace("@FORM@", &format!(".{}", SEARCH.class()))
        .replace("@RESULTS@", &format!(".{}", SEARCH.element("results")))
        .replace("@RESULT@", &SEARCH.element("result"))
}

const ROUTER: &str = include_str!("router.js");
const SEARCH_CLIENT: &str = include_str!("search.js");

#[cfg(test)]
mod tests {
    use crabbucket::style::StyleSheet;

    use super::components::{CALLOUT, Callout, Kind, callout};
    use super::{DOCS, MASTHEAD, NS, PROSE, SITE};

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
        let props = Callout {
            kind: Kind::Warn,
            title: None,
        };
        let markup = callout(&props, maud::html! { p { "hi" } }).into_string();
        assert!(markup.contains("cb-callout cb-callout--warn"));
        assert!(markup.contains("cb-callout__body"));

        let css = CALLOUT.render();
        assert!(css.contains(".cb-callout--warn"));
        assert!(css.contains(".cb-callout__body"));
    }
}
