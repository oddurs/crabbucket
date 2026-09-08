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
//! Where a route comes from, and how to turn a set of them into a type.
//!
//! This crate answers one question -- what route does this file have? -- and
//! it is the only place that answers it.  `crabbucket` uses it at run time to
//! decide where a page is written; a site's `build.rs` uses it at build time
//! to generate a `Route` enum, so that a link written in Rust stops compiling
//! when the page it points at is renamed.
//!
//! It has no dependencies.  It is compiled during a site's build phase, before
//! anything else, and pulling a Markdown parser and a syntax highlighter in
//! there to walk a directory would be absurd.
//!
//! ```
//! use crabbucket_routes::{generate, route_of};
//! use std::path::Path;
//!
//! assert_eq!(route_of(Path::new("content/docs/routing.md"), Path::new("content")), "docs/routing");
//! assert_eq!(route_of(Path::new("content/index.md"), Path::new("content")), "");
//!
//! let module = generate(&["".into(), "docs/routing".into()]).unwrap();
//! assert!(module.contains("Index,"));
//! assert!(module.contains("DocsRouting,"));
//! ```

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The extension a content file has.
const CONTENT: &str = "md";

/// Something wrong with a set of routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// Derives a route from a content file's path below the content root.
///
/// A file's path is its route, with the extension dropped; any `index.md`
/// takes the route of its directory, and the content root's own `index.md` is
/// the empty route, which is the site root.
pub fn route_of(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path).with_extension("");
    let mut route = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    if route == "index" {
        route.clear();
    } else if let Some(parent) = route.strip_suffix("/index") {
        route = parent.to_string();
    }

    route
}

/// Every route below `root`, sorted, with the files they came from.
///
/// # Errors
///
/// Fails if the directory cannot be read.
pub fn scan(root: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let mut found = Vec::new();
    walk(root, root, &mut found)?;
    found.sort();
    Ok(found)
}

fn walk(dir: &Path, root: &Path, found: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            walk(&path, root, found)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == CONTENT)
        {
            found.push((route_of(&path, root), path));
        }
    }

    Ok(())
}

/// The `cargo::rerun-if-changed` lines a site's build script should print.
///
/// The content directory itself is listed as well as every file in it, because
/// cargo only notices a *new* file through the directory, and a build script
/// that watches the files alone silently misses the page you just added.
pub fn watch(root: &Path, found: &[(String, PathBuf)]) -> String {
    let mut out = format!("cargo::rerun-if-changed={}\n", root.display());

    for (_, path) in found {
        out.push_str(&format!("cargo::rerun-if-changed={}\n", path.display()));
    }

    out
}

/// Generates the `Route` enum for a set of routes.
///
/// # Errors
///
/// Fails if two routes would produce the same variant name, which is the one
/// way this can silently go wrong.
pub fn generate(routes: &[String]) -> Result<String, Error> {
    // Declared in route order, so `Route::ALL` reads the way a sitemap should
    // and the empty route -- the site root -- comes first.
    let mut named: Vec<(String, &String)> =
        routes.iter().map(|route| (variant(route), route)).collect();

    named.sort_by_key(|(_, route)| *route);

    // Collisions are found in name order, which is the order they collide in.
    let mut by_name = named.clone();
    by_name.sort();

    for pair in by_name.windows(2) {
        let [(name, first), (other, second)] = pair else {
            continue;
        };

        if name == other {
            return Err(Error(format!(
                "`{first}` and `{second}` would both be `Route::{name}`; \
                 rename one of them"
            )));
        }
    }

    let mut out = String::from(
        "// Generated by crabbucket-routes.  Do not edit.\n\n\
         /// Every route in `content/`, as a type.\n\
         ///\n\
         /// A link to a page that does not exist does not compile.  Rename a\n\
         /// page and every reference to it stops building, which is the whole\n\
         /// reason this is generated rather than written.\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum Route {\n",
    );

    for (name, route) in &named {
        out.push_str(&format!("    /// `/{route}`\n    {name},\n"));
    }

    out.push_str("}\n\nimpl Route {\n    /// Every route, in name order.\n");
    out.push_str("    pub const ALL: &'static [Route] = &[\n");

    for (name, _) in &named {
        out.push_str(&format!("        Route::{name},\n"));
    }

    out.push_str("    ];\n\n");
    out.push_str(
        "    /// The site-relative path, without the base.\n    \
         pub const fn path(self) -> &'static str {\n        match self {\n",
    );

    for (name, route) in &named {
        out.push_str(&format!("            Route::{name} => {route:?},\n"));
    }

    out.push_str("        }\n    }\n}\n");
    Ok(out)
}

/// Turns a route into a variant name: `docs/getting-started` becomes
/// `DocsGettingStarted`, and the empty route becomes `Index`.
fn variant(route: &str) -> String {
    let route = route.trim_matches('/');

    if route.is_empty() {
        return "Index".to_string();
    }

    let mut out = String::with_capacity(route.len());
    let mut capitalise = true;

    for character in route.chars() {
        if character.is_alphanumeric() {
            if capitalise {
                out.extend(character.to_uppercase());
                capitalise = false;
            } else {
                out.push(character);
            }
        } else {
            capitalise = true;
        }
    }

    // A route starting with a digit would not be an identifier.
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'R');
    }

    out
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{generate, route_of, variant, watch};

    #[test]
    fn a_path_below_the_root_is_a_route() {
        let root = Path::new("content");

        assert_eq!(route_of(Path::new("content/index.md"), root), "");
        assert_eq!(route_of(Path::new("content/about.md"), root), "about");
        assert_eq!(
            route_of(Path::new("content/docs/intro.md"), root),
            "docs/intro"
        );
        assert_eq!(route_of(Path::new("content/docs/index.md"), root), "docs");
    }

    #[test]
    fn a_variant_name_is_the_route_in_camel_case() {
        assert_eq!(variant(""), "Index");
        assert_eq!(variant("about"), "About");
        assert_eq!(variant("docs/getting-started"), "DocsGettingStarted");
        assert_eq!(variant("a/b/c"), "ABC");
    }

    #[test]
    fn a_route_that_would_not_be_an_identifier_becomes_one() {
        assert_eq!(variant("2026/notes"), "R2026Notes");
        assert_eq!(variant("docs/v1.0"), "DocsV10");
    }

    #[test]
    fn routes_are_declared_in_path_order_with_the_root_first() {
        let routes = vec![
            "docs/routing".to_string(),
            String::new(),
            "about".to_string(),
        ];
        let module = generate(&routes).unwrap();

        let index = module.find("    Index,").expect("no Index");
        let about = module.find("    About,").expect("no About");
        let routing = module.find("    DocsRouting,").expect("no DocsRouting");

        assert!(
            index < about && about < routing,
            "declared out of order:\n{module}"
        );
    }

    #[test]
    fn the_generated_enum_carries_every_route_and_its_path() {
        let routes = vec![
            String::new(),
            "docs".to_string(),
            "docs/routing".to_string(),
        ];
        let module = generate(&routes).unwrap();

        assert!(module.contains("    Index,"), "got {module}");
        assert!(module.contains("    DocsRouting,"));
        assert!(module.contains("Route::Index => \"\","), "got {module}");
        assert!(module.contains("Route::DocsRouting => \"docs/routing\","));
        assert!(module.contains("pub const ALL: &'static [Route]"));
    }

    #[test]
    fn every_variant_is_documented_with_the_url_it_stands_for() {
        let module = generate(&["docs/routing".to_string()]).unwrap();
        assert!(module.contains("/// `/docs/routing`"), "got {module}");
    }

    #[test]
    fn two_routes_that_would_share_a_name_fail_rather_than_one_winning() {
        // `docs/getting-started` and `docs/getting/started` both camel-case to
        // `DocsGettingStarted`.  Picking one silently would mean a link
        // compiling and going to the wrong page.
        let routes = vec![
            "docs/getting-started".to_string(),
            "docs/getting/started".to_string(),
        ];
        let err = generate(&routes).unwrap_err();

        assert!(err.to_string().contains("DocsGettingStarted"), "got {err}");
        assert!(
            err.to_string().contains("docs/getting-started"),
            "it names both: {err}"
        );
        assert!(
            err.to_string().contains("docs/getting/started"),
            "it names both: {err}"
        );
    }

    #[test]
    fn an_empty_site_still_generates_something_that_compiles() {
        let module = generate(&[]).unwrap();
        assert!(module.contains("pub enum Route {"));
        assert!(
            module.contains("pub const ALL: &'static [Route] = &[\n    ];"),
            "got {module}"
        );
    }

    #[test]
    fn the_content_directory_is_watched_as_well_as_the_files_in_it() {
        // Cargo only notices a *new* file through the directory.  A build
        // script watching the files alone misses the page you just added,
        // which is the classic build-script bug.
        let found = vec![(
            "about".to_string(),
            Path::new("content/about.md").to_path_buf(),
        )];
        let lines = watch(Path::new("content"), &found);

        assert!(
            lines.contains("cargo::rerun-if-changed=content\n"),
            "got {lines}"
        );
        assert!(
            lines.contains("cargo::rerun-if-changed=content/about.md\n"),
            "got {lines}"
        );
    }
}
