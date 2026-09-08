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
//! Swapping this crate for another one is how a site changes design system.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crabbucket::theme::{NavItem, Page, Theme};
use crabbucket::{Config, Url};
use maud::{DOCTYPE, Markup, PreEscaped, html};

/// Design tokens, generated from `design/tokens.toml`.
///
/// Each constant is a `var(--group-name, fallback)` reference, so a component
/// picks up a theme override at runtime while still failing to compile if the
/// token itself is renamed.
pub mod tok {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

/// The default design system.
///
/// A site swaps design system by handing [`crabbucket::build`] a different
/// implementation of [`Theme`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Standard;

/// The kind of a callout, which decides its accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        aside class={ "callout callout--" (kind.slug()) } {
            div."callout__body" { (body) }
        }
    }
}

/// The whole document: head, navigation, content, footer.
pub fn document(config: &Config, title: &str, nav: &[NavItem], content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — " (config.title) }
                @if !config.description.is_empty() {
                    meta name="description" content=(config.description);
                }
                link rel="stylesheet" href=(Url::asset(config, "site.css"));
                @if config.router {
                    script defer src=(Url::asset(config, "router.js")) {}
                }
            }
            body {
                header."masthead" {
                    a."masthead__home" href=(Url::new(config, "")) { (config.title) }
                    nav."masthead__nav" {
                        @for item in nav {
                            a href=(item.href)
                              aria-current=[item.current.then_some("page")] { (item.label) }
                        }
                    }
                }
                main."page" { (content) }
                footer."colophon" {
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

impl Theme for Standard {
    fn render(&self, page: &Page<'_>) -> String {
        let title = &page.meta.title;
        document(page.config, title, page.nav, prose(page.html)).into_string()
    }

    fn stylesheet(&self) -> String {
        stylesheet()
    }

    fn router_js(&self) -> &str {
        router_js()
    }
}

/// Renders a body of Markdown-derived HTML inside the prose wrapper.
///
/// The HTML comes from the site's own content, which is trusted, so it is
/// emitted unescaped.
pub fn prose(html_fragment: &str) -> Markup {
    html! { div."prose" { (PreEscaped(html_fragment)) } }
}

/// The site's complete stylesheet: the generated token block, then the base
/// styles that consume it.
pub fn stylesheet() -> String {
    format!("{}\n{}", tok::CSS, BASE)
}

/// The client-side router, as described in section 7 of `doc/DESIGN`.
///
/// It intercepts same-origin, same-document link clicks, fetches the target,
/// swaps `<main>` and the title, and drives the View Transitions API when the
/// browser has one.  With JavaScript off, or on a browser that fails any of
/// its guards, navigation is what it always was.
pub fn router_js() -> &'static str {
    ROUTER
}

const BASE: &str = r#"
*, *::before, *::after { box-sizing: border-box; }
html { color-scheme: dark; }
body {
  margin: 0;
  background: var(--color-surface);
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--size-step-0);
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}
a { color: var(--color-link); text-underline-offset: 0.2em; }
main.page { max-width: var(--measure-page); margin: 0 auto; padding: var(--space-xl) var(--space-lg); }
.prose { max-width: var(--measure-prose); }
.prose h1 { font-size: var(--size-step-4); line-height: 1.1; letter-spacing: -0.02em; margin: 0 0 var(--space-md); }
.prose h2 { font-size: var(--size-step-2); line-height: 1.2; margin: var(--space-xl) 0 var(--space-sm); }
.prose h3 { font-size: var(--size-step-1); margin: var(--space-lg) 0 var(--space-xs); }
.prose p, .prose ul, .prose ol { margin: 0 0 var(--space-md); }
.prose code { font-family: var(--font-mono); font-size: var(--size-step--1); background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: var(--radius-sm); padding: 0.1em 0.35em; }
.prose pre { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: var(--radius-md); padding: var(--space-md); overflow-x: auto; }
.prose pre code { background: none; border: 0; padding: 0; }
.masthead { display: flex; gap: var(--space-lg); align-items: baseline; justify-content: space-between; flex-wrap: wrap; max-width: var(--measure-page); margin: 0 auto; padding: var(--space-lg); border-bottom: 1px solid var(--color-border); }
.masthead__home { font-weight: 600; color: var(--color-text); text-decoration: none; }
.masthead__nav { display: flex; gap: var(--space-md); }
.masthead__nav a { color: var(--color-text-muted); text-decoration: none; }
.masthead__nav a[aria-current="page"] { color: var(--color-accent); }
.callout { border-left: 3px solid var(--color-accent); background: var(--color-surface-raised); border-radius: 0 var(--radius-md) var(--radius-md) 0; padding: var(--space-md); margin: 0 0 var(--space-md); }
.callout--note { border-left-color: var(--color-link); }
.callout--warn { border-left-color: var(--color-accent); }
.callout__body > :last-child { margin-bottom: 0; }
.colophon { max-width: var(--measure-page); margin: 0 auto; padding: var(--space-lg); border-top: 1px solid var(--color-border); color: var(--color-text-muted); font-size: var(--size-step--1); }
@media (prefers-reduced-motion: reduce) { ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; } }
"#;

const ROUTER: &str = r#"// crabbucket client router. GPL-3.0-or-later.
(() => {
  const swap = (doc) => {
    document.querySelector('main').replaceWith(doc.querySelector('main'));
    document.title = doc.title;
    document.querySelectorAll('.masthead__nav a').forEach((a) => {
      a.toggleAttribute('aria-current', a.pathname === location.pathname);
    });
  };
  const go = async (url, push) => {
    const res = await fetch(url, { headers: { 'x-crabbucket': '1' } });
    if (!res.ok) { location.assign(url); return; }
    const doc = new DOMParser().parseFromString(await res.text(), 'text/html');
    if (push) history.pushState(null, '', url);
    document.startViewTransition ? document.startViewTransition(() => swap(doc)) : swap(doc);
    scrollTo(0, 0);
  };
  addEventListener('click', (e) => {
    if (e.defaultPrevented || e.button || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const a = e.target.closest('a');
    if (!a || a.target || a.hasAttribute('download') || a.origin !== location.origin) return;
    if (a.pathname === location.pathname) return;
    e.preventDefault();
    go(a.href, true).catch(() => location.assign(a.href));
  });
  addEventListener('popstate', () => go(location.href, false).catch(() => location.reload()));
})();
"#;
