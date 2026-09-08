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
//! Link checking, which is a build gate rather than a lint.
//!
//! Content is Markdown, so a link written in a page cannot be typed the way a
//! link written in a component can.  The next best thing is to make a dead
//! internal link fail the build rather than fail the reader, so that the
//! promise in `doc/DESIGN` -- the build either fails, or the site is correct --
//! holds for the half of the site that has no compiler.
//!
//! External links are not checked.  Their liveness is not a property of this
//! build, and a generator that phones out to the network to decide whether it
//! succeeded is a generator that fails on an aeroplane.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// A link that goes nowhere, and the page it was written in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeadLink {
    /// The content file that contains the link.
    pub source: PathBuf,
    /// The link exactly as written.
    pub href: String,
    /// Where it resolved to, which is what makes the diagnosis obvious.
    pub target: String,
}

/// One rendered page, awaiting checking.
pub struct Rendered<'a> {
    /// The content file it came from, for diagnostics.
    pub source: &'a Path,
    /// The URL the page is served at, which relative links resolve against.
    pub url: &'a str,
    /// The rendered HTML.
    pub html: &'a str,
}

/// The outcome of a check: what was examined, and what was dead.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Check {
    /// Every dead internal link, sorted and deduplicated.
    pub dead: Vec<DeadLink>,
    /// How many internal links were resolved.  External links, bare fragments
    /// and anything outside the base path are not counted, because they were
    /// not checked.
    pub examined: usize,
}

/// Checks every internal link in every page.
///
/// A link resolving to a directory is checked against `routes`; a link
/// resolving to a file is checked against `assets`.  Anything external, or any
/// bare fragment, is left alone.
pub fn check(
    config: &Config,
    pages: &[Rendered<'_>],
    routes: &BTreeSet<String>,
    assets: &BTreeSet<String>,
) -> Check {
    let mut dead = Vec::new();
    let mut examined = 0;

    for page in pages {
        for href in hrefs(page.html) {
            let Some(target) = resolve(config, page.url, &href) else {
                continue;
            };

            examined += 1;

            let ok = match target.strip_suffix('/') {
                Some(route) => routes.contains(route.trim_matches('/')),
                None => assets.contains(target.trim_start_matches('/')),
            };

            if !ok {
                dead.push(DeadLink {
                    source: page.source.to_path_buf(),
                    href: href.clone(),
                    target,
                });
            }
        }
    }

    dead.sort();
    dead.dedup();
    Check { dead, examined }
}

/// Pulls every `href` and `src` value out of a document.
///
/// A real parser is not needed and would not help: the input is this crate's
/// own output, where every attribute is emitted by `maud` in one form.
fn hrefs(html: &str) -> Vec<String> {
    let mut found = Vec::new();

    for attribute in ["href=\"", "src=\""] {
        let mut rest = html;
        while let Some(start) = rest.find(attribute) {
            rest = &rest[start + attribute.len()..];
            match rest.find('"') {
                Some(end) => {
                    found.push(rest[..end].to_string());
                    rest = &rest[end + 1..];
                }
                None => break,
            }
        }
    }

    found
}

/// Resolves a link against the page it appears in.
///
/// Returns `None` for anything that is not this site's problem: another
/// origin, a scheme, a bare fragment, an empty href.
fn resolve(config: &Config, page_url: &str, href: &str) -> Option<String> {
    let href = href.trim();

    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("//")
        || href.contains("://")
        || href.starts_with("mailto:")
        || href.starts_with("data:")
        || href.starts_with("tel:")
    {
        return None;
    }

    // A fragment or a query is not part of what is on disk.
    let path = href.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return None;
    }

    let joined = if let Some(absolute) = path.strip_prefix('/') {
        format!("/{absolute}")
    } else {
        // Relative links resolve against the page's directory, and a page URL
        // always ends in a slash because every route is a directory.
        format!("{page_url}{path}")
    };

    let trailing = joined.ends_with('/');
    let mut parts: Vec<&str> = Vec::new();

    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }

    let mut out = format!("/{}", parts.join("/"));
    if trailing && !out.ends_with('/') {
        out.push('/');
    }

    // Everything below the base path is the site; anything above it is not
    // something this build can vouch for.
    let base = config.base.trim_end_matches('/');
    let relative = out.strip_prefix(base)?;
    let relative = if relative.is_empty() { "/" } else { relative };

    Some(relative.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::{Rendered, check, hrefs, resolve};
    use crate::config::Config;

    fn config(base: &str) -> Config {
        Config {
            title: "t".into(),
            description: String::new(),
            url: None,
            base: base.into(),
            router: false,
        }
    }

    #[test]
    fn both_attributes_are_collected() {
        let html = r#"<a href="/a/">a</a><img src="/b.png"><a href="c/">c</a>"#;
        assert_eq!(hrefs(html), ["/a/", "c/", "/b.png"]);
    }

    #[test]
    fn absolute_links_lose_the_base_path() {
        let config = config("/repo/");
        assert_eq!(
            resolve(&config, "/repo/", "/repo/docs/").as_deref(),
            Some("/docs/")
        );
        assert_eq!(
            resolve(&config, "/repo/", "/repo/site.css").as_deref(),
            Some("/site.css")
        );
    }

    #[test]
    fn relative_links_resolve_against_the_page() {
        let config = config("/repo/");
        let from_docs = |href| resolve(&config, "/repo/docs/", href);
        assert_eq!(from_docs("content/").as_deref(), Some("/docs/content/"));
        assert_eq!(from_docs("../about/").as_deref(), Some("/about/"));
        assert_eq!(from_docs("./content/").as_deref(), Some("/docs/content/"));
    }

    #[test]
    fn fragments_and_queries_are_stripped_before_resolving() {
        let config = config("/");
        assert_eq!(
            resolve(&config, "/", "/docs/#install").as_deref(),
            Some("/docs/")
        );
        assert_eq!(
            resolve(&config, "/", "/docs/?q=1").as_deref(),
            Some("/docs/")
        );
    }

    #[test]
    fn what_is_not_ours_is_left_alone() {
        let config = config("/");
        for href in [
            "",
            "#top",
            "https://example.com/",
            "//example.com/",
            "mailto:a@b.c",
        ] {
            assert_eq!(resolve(&config, "/", href), None, "for href {href:?}");
        }
    }

    #[test]
    fn a_link_above_the_base_path_is_not_ours_either() {
        let config = config("/repo/");
        assert_eq!(resolve(&config, "/repo/", "/elsewhere/"), None);
    }

    #[test]
    fn a_dead_route_is_reported_with_the_page_that_wrote_it() {
        let config = config("/repo/");
        let routes = BTreeSet::from(["".to_string(), "docs".to_string()]);
        let assets = BTreeSet::from(["site.css".to_string()]);
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/repo/",
            html: r#"<a href="/repo/docs/">ok</a><a href="/repo/gone/">dead</a>"#,
        }];

        let result = check(&config, &pages, &routes, &assets);
        assert_eq!(result.examined, 2);
        assert_eq!(result.dead.len(), 1);
        assert_eq!(result.dead[0].href, "/repo/gone/");
        assert_eq!(result.dead[0].target, "/gone/");
        assert_eq!(result.dead[0].source, Path::new("content/index.md"));
    }

    #[test]
    fn a_live_site_reports_nothing() {
        let config = config("/");
        let routes = BTreeSet::from(["".to_string(), "docs".to_string()]);
        let assets = BTreeSet::from(["site.css".to_string()]);
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/",
            html: r#"<link href="/site.css"><a href="/docs/">d</a><a href="https://x.example/">x</a>"#,
        }];

        let result = check(&config, &pages, &routes, &assets);
        assert!(result.dead.is_empty());
        assert_eq!(result.examined, 2, "the external link is not checked");
    }
}
