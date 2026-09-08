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
//! Fragments are checked too, against the heading ids the Markdown pass
//! collected.  A renamed heading is the more common failure of the two, since
//! headings get reworded far more often than pages get renamed.
//!
//! External links are not checked.  Their liveness is not a property of this
//! build, and a generator that phones out to the network to decide whether it
//! succeeded is a generator that fails on an aeroplane.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Why a link is dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reason {
    /// Nothing is served at that path.
    NoSuchRoute,
    /// No such file will be written.
    NoSuchAsset,
    /// The page exists, but has no heading with that id.
    NoSuchAnchor,
    /// A relative link on the error page, which is served from every path and
    /// so has no directory to be relative to.
    RelativeOnErrorPage,
}

impl Reason {
    fn describe(self, target: &str) -> String {
        match self {
            Reason::NoSuchRoute => format!("{target} is not a route"),
            Reason::NoSuchAsset => format!("{target} is not a file this build writes"),
            Reason::NoSuchAnchor => format!("{target} has no such heading"),
            Reason::RelativeOnErrorPage => format!(
                "{target} is relative, and the error page is served from every path; \
                 write it as a site-absolute link"
            ),
        }
    }
}

/// A link that goes nowhere, and the page it was written in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeadLink {
    /// The content file that contains the link.
    pub source: PathBuf,
    /// The link exactly as written.
    pub href: String,
    /// Where it resolved to, which is what makes the diagnosis obvious.
    pub target: String,
    /// What is wrong with it.
    pub reason: Reason,
}

impl DeadLink {
    /// The one-line explanation, without the source path.
    pub fn describe(&self) -> String {
        format!("{} -> {}", self.href, self.reason.describe(&self.target))
    }
}

/// One rendered page, awaiting checking.
pub struct Rendered<'a> {
    /// The content file it came from, for diagnostics.
    pub source: &'a Path,
    /// The URL the page is served at, which relative links resolve against.
    pub url: &'a str,
    /// The rendered HTML.
    pub html: &'a str,
    /// The heading ids on this page, for fragments that point into it.
    ///
    /// A bare `#fragment` is checked against this rather than through the
    /// route table, because a page is allowed to have anchors without being a
    /// route -- which is exactly the error page's situation.
    pub anchors: &'a BTreeSet<String>,
    /// Whether this is the error page.  It is served in place of any missing
    /// path, so it has no directory for a relative link to resolve against.
    pub error_page: bool,
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

/// Every route the build will write, and the heading ids on each.
pub type Routes = BTreeMap<String, BTreeSet<String>>;

/// Checks every internal link in every page.
///
/// A link resolving to a directory is checked against `routes`; a link
/// resolving to a file is checked against `assets`; a fragment is checked
/// against the target page's headings.  Anything external is left alone.
pub fn check(
    config: &Config,
    pages: &[Rendered<'_>],
    routes: &Routes,
    assets: &BTreeSet<String>,
) -> Check {
    let mut dead = Vec::new();
    let mut examined = 0;

    for page in pages {
        for href in hrefs(page.html) {
            let Some(link) = resolve(config, page.url, &href) else {
                continue;
            };

            examined += 1;

            if page.error_page && is_relative(&href) {
                dead.push(DeadLink {
                    source: page.source.to_path_buf(),
                    href: href.clone(),
                    target: href.clone(),
                    reason: Reason::RelativeOnErrorPage,
                });
                continue;
            }

            // A bare fragment points into the page it was written in, whether
            // or not that page is a route.
            if href.starts_with('#') {
                if let Some(anchor) = &link.fragment
                    && !page.anchors.contains(anchor)
                {
                    dead.push(DeadLink {
                        source: page.source.to_path_buf(),
                        href: href.clone(),
                        target: format!("{}#{anchor}", link.path),
                        reason: Reason::NoSuchAnchor,
                    });
                }
                continue;
            }

            let reason = match link.path.strip_suffix('/') {
                Some(route) => {
                    let route = route.trim_matches('/');
                    match routes.get(route) {
                        None => Some(Reason::NoSuchRoute),
                        Some(anchors) => match &link.fragment {
                            Some(anchor) if !anchors.contains(anchor) => Some(Reason::NoSuchAnchor),
                            _ => None,
                        },
                    }
                }
                None => (!assets.contains(link.path.trim_start_matches('/')))
                    .then_some(Reason::NoSuchAsset),
            };

            if let Some(reason) = reason {
                dead.push(DeadLink {
                    source: page.source.to_path_buf(),
                    href: href.clone(),
                    target: match (&link.fragment, reason) {
                        (Some(anchor), Reason::NoSuchAnchor) => {
                            format!("{}#{anchor}", link.path)
                        }
                        _ => link.path.clone(),
                    },
                    reason,
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

/// Whether an href is relative, and so depends on where the page was served.
fn is_relative(href: &str) -> bool {
    let href = href.trim();
    !href.is_empty()
        && !href.starts_with('/')
        && !href.starts_with('#')
        && !href.contains("://")
        && !href.starts_with("mailto:")
        && !href.starts_with("tel:")
        && !href.starts_with("data:")
}

/// A link, resolved against the page it appears in.
#[derive(Debug, PartialEq, Eq)]
struct Link {
    /// The site-relative path, with the base stripped.
    path: String,
    /// The fragment, if the link carried one.
    fragment: Option<String>,
}

/// Resolves a link against the page it appears in.
///
/// Returns `None` for anything that is not this site's problem: another
/// origin, a scheme, an empty href, or a bare `#`, which is the conventional
/// spelling of "no destination" rather than a broken link.
fn resolve(config: &Config, page_url: &str, href: &str) -> Option<Link> {
    let href = href.trim();

    if href.is_empty()
        || href == "#"
        || href.starts_with("//")
        || href.contains("://")
        || href.starts_with("mailto:")
        || href.starts_with("data:")
        || href.starts_with("tel:")
    {
        return None;
    }

    let (path, fragment) = split_fragment(href);

    // A bare fragment points into the page it was written in.
    let joined = if path.is_empty() {
        page_url.to_string()
    } else if let Some(absolute) = path.strip_prefix('/') {
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

    Some(Link {
        path: relative.to_string(),
        fragment,
    })
}

/// Splits an href into its path and its fragment, dropping any query string,
/// which is not part of what is on disk.
fn split_fragment(href: &str) -> (&str, Option<String>) {
    let (before, fragment) = match href.split_once('#') {
        Some((before, fragment)) if !fragment.is_empty() => (before, Some(fragment.to_string())),
        Some((before, _)) => (before, None),
        None => (href, None),
    };

    (before.split('?').next().unwrap_or(""), fragment)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use super::{Link, Reason, Rendered, Routes, check, hrefs, resolve};
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

    fn routes(pairs: &[(&str, &[&str])]) -> Routes {
        pairs
            .iter()
            .map(|(route, anchors)| {
                (
                    route.to_string(),
                    anchors.iter().map(|a| a.to_string()).collect(),
                )
            })
            .collect()
    }

    fn anchors(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn assets(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn path(config: &Config, page: &str, href: &str) -> Option<String> {
        resolve(config, page, href).map(|link| link.path)
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
            path(&config, "/repo/", "/repo/docs/").as_deref(),
            Some("/docs/")
        );
        assert_eq!(
            path(&config, "/repo/", "/repo/site.css").as_deref(),
            Some("/site.css")
        );
    }

    #[test]
    fn relative_links_resolve_against_the_page() {
        let config = config("/repo/");
        let from_docs = |href| path(&config, "/repo/docs/", href);
        assert_eq!(from_docs("content/").as_deref(), Some("/docs/content/"));
        assert_eq!(from_docs("../about/").as_deref(), Some("/about/"));
        assert_eq!(from_docs("./content/").as_deref(), Some("/docs/content/"));
    }

    #[test]
    fn a_fragment_is_kept_and_a_query_is_not() {
        let config = config("/");
        assert_eq!(
            resolve(&config, "/", "/docs/#install"),
            Some(Link {
                path: "/docs/".into(),
                fragment: Some("install".into())
            })
        );
        assert_eq!(
            resolve(&config, "/", "/docs/?q=1"),
            Some(Link {
                path: "/docs/".into(),
                fragment: None
            })
        );
    }

    #[test]
    fn a_bare_fragment_points_into_its_own_page() {
        let config = config("/repo/");
        assert_eq!(
            resolve(&config, "/repo/docs/", "#install"),
            Some(Link {
                path: "/docs/".into(),
                fragment: Some("install".into())
            })
        );
    }

    #[test]
    fn what_is_not_ours_is_left_alone() {
        let config = config("/");
        for href in [
            "",
            "#",
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
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/repo/",
            anchors: &BTreeSet::new(),
            error_page: false,
            html: r#"<a href="/repo/docs/">ok</a><a href="/repo/gone/">dead</a>"#,
        }];

        let result = check(
            &config,
            &pages,
            &routes(&[("", &[]), ("docs", &[])]),
            &assets(&["site.css"]),
        );

        assert_eq!(result.examined, 2);
        assert_eq!(result.dead.len(), 1);
        assert_eq!(result.dead[0].href, "/repo/gone/");
        assert_eq!(result.dead[0].target, "/gone/");
        assert_eq!(result.dead[0].reason, Reason::NoSuchRoute);
        assert_eq!(result.dead[0].source, Path::new("content/index.md"));
    }

    #[test]
    fn a_missing_asset_is_reported_as_a_file_and_not_as_a_route() {
        let config = config("/");
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/",
            anchors: &BTreeSet::new(),
            error_page: false,
            html: r#"<img src="/logo.png">"#,
        }];

        let result = check(
            &config,
            &pages,
            &routes(&[("", &[])]),
            &assets(&["site.css"]),
        );
        assert_eq!(result.dead[0].reason, Reason::NoSuchAsset);
    }

    #[test]
    fn a_fragment_naming_no_heading_is_dead() {
        let config = config("/");
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/",
            anchors: &BTreeSet::new(),
            error_page: false,
            html: r#"<a href="/docs/#install">a</a><a href="/docs/#gone">b</a>"#,
        }];

        let result = check(
            &config,
            &pages,
            &routes(&[("", &[]), ("docs", &["install"])]),
            &assets(&[]),
        );

        assert_eq!(result.dead.len(), 1);
        assert_eq!(result.dead[0].href, "/docs/#gone");
        assert_eq!(result.dead[0].target, "/docs/#gone");
        assert_eq!(result.dead[0].reason, Reason::NoSuchAnchor);
    }

    #[test]
    fn a_same_page_fragment_is_checked_against_that_page() {
        let config = config("/");
        let here = anchors(&["install"]);
        let pages = [Rendered {
            source: Path::new("content/docs.md"),
            url: "/docs/",
            anchors: &here,
            error_page: false,
            html: r##"<a href="#install">a</a><a href="#gone">b</a>"##,
        }];

        let result = check(
            &config,
            &pages,
            &routes(&[("docs", &["install"])]),
            &assets(&[]),
        );
        assert_eq!(result.dead.len(), 1);
        assert_eq!(result.dead[0].href, "#gone");
    }

    #[test]
    fn a_fragment_on_an_external_link_is_not_our_business() {
        let config = config("/");
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/",
            anchors: &BTreeSet::new(),
            error_page: false,
            html: r#"<a href="https://example.com/x/#nope">x</a>"#,
        }];

        let result = check(&config, &pages, &routes(&[("", &[])]), &assets(&[]));
        assert!(result.dead.is_empty());
        assert_eq!(result.examined, 0);
    }

    #[test]
    fn a_live_site_reports_nothing() {
        let config = config("/");
        let pages = [Rendered {
            source: Path::new("content/index.md"),
            url: "/",
            anchors: &BTreeSet::new(),
            error_page: false,
            html: r#"<link href="/site.css"><a href="/docs/">d</a><a href="https://x.example/">x</a>"#,
        }];

        let result = check(
            &config,
            &pages,
            &routes(&[("", &[]), ("docs", &[])]),
            &assets(&["site.css"]),
        );

        assert!(result.dead.is_empty());
        assert_eq!(result.examined, 2, "the external link is not checked");
    }
}
