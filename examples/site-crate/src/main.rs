//! Builds this site.
//!
//! The interesting part is `routes`: a link written here is checked by the
//! compiler, which is the half of link safety that content -- having no
//! compiler -- cannot have.

use std::path::Path;
use std::process::ExitCode;

use crabbucket::{Config, Url};
use crabbucket_theme_plain::Plain;
use maud::{Markup, html};

/// Generated from `content/` by `build.rs`.
pub mod routes {
    include!(concat!(env!("OUT_DIR"), "/routes.rs"));
}

use routes::Route;

/// A link to a route, which cannot point at a page that does not exist.
pub fn link(config: &Config, route: Route, label: &str) -> Markup {
    html! { a href=(Url::new(config, route.path())) { (label) } }
}

/// Every route the site has, which is a list nobody maintains.
pub fn sitemap(config: &Config) -> Markup {
    html! {
        ul {
            @for route in Route::ALL {
                li { (link(config, *route, route.path())) }
            }
        }
    }
}

/// A link written by name rather than by string.
///
/// This is the whole demonstration: rename `content/docs/routing.md` and this
/// function stops compiling.  `crates/crabbucket-routes/tests/rename.rs`
/// asserts exactly that, by doing it.
pub fn read_more(config: &Config) -> Markup {
    link(config, Route::DocsRouting, "The route table")
}

fn main() -> ExitCode {
    match crabbucket::build(Path::new(env!("CARGO_MANIFEST_DIR")), &Plain) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!(
                "crabbucket-example-site: {}",
                err.render(std::io::IsTerminal::is_terminal(&std::io::stderr()))
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, link};
    use crabbucket::Config;

    fn config() -> Config {
        Config {
            title: "t".into(),
            description: String::new(),
            url: None,
            base: "/repo/".into(),
            search: false,
            feeds: Vec::new(),
            router: false,
        }
    }

    #[test]
    fn the_enum_has_a_variant_for_every_page() {
        // If a page is added or renamed, this list changes, and so does every
        // link written by name.
        let paths: Vec<&str> = Route::ALL.iter().map(|route| route.path()).collect();
        assert_eq!(
            paths,
            [
                "",
                "docs",
                "docs/routing",
                "notes",
                "notes/one-implementation",
                "notes/typed-routes"
            ]
        );
    }

    #[test]
    fn a_typed_link_carries_the_base_path_like_any_other() {
        let html = link(&config(), Route::DocsRouting, "The route table").into_string();
        assert_eq!(html, "<a href=\"/repo/docs/routing/\">The route table</a>");
    }

    #[test]
    fn the_root_route_is_the_base_itself() {
        let html = link(&config(), Route::Index, "Home").into_string();
        assert_eq!(html, "<a href=\"/repo/\">Home</a>");
    }
}
