//! Builds this site.
//!
//! The interesting part is `routes`: a link written here is checked by the
//! compiler, which is the half of link safety that content -- having no
//! compiler -- cannot have.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use crabbucket::directive::Directives;
use crabbucket::site::Options;
use crabbucket::{Config, Url};
use crabbucket_theme_plain::Plain;
use maud::{Markup, html};
use serde::Deserialize;

/// Generated from `content/` by `build.rs`.
pub mod routes {
    include!(concat!(env!("OUT_DIR"), "/routes.rs"));
}

use routes::Route;

/// The recordings in `data/recordings.toml`.
///
/// The site's own type, deserialized from the site's own file. crabbucket
/// knows nothing about either.
#[derive(Debug, Deserialize)]
struct Recordings {
    #[serde(flatten)]
    by_name: BTreeMap<String, Recording>,
}

#[derive(Debug, Deserialize)]
struct Recording {
    command: String,
    output: String,
}

/// `:::terminal{name = "build"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Terminal {
    name: String,
    #[serde(default)]
    caption: Option<String>,
}

/// The site's own directives, which its design system knows nothing about.
///
/// A `terminal` belongs to this site: it reads recordings this site generates.
/// No design system should have to know that, and before `Options::directives`
/// there was nowhere for it to live.
fn directives() -> Directives {
    let mut directives = Directives::new();

    directives.add_with("terminal", |props: Terminal, _, context| {
        let recordings: Recordings = context.data("recordings")?;

        let recording = recordings
            .by_name
            .get(&props.name)
            .ok_or_else(|| format!("there is no recording called `{}`", props.name))?;

        Ok(html! {
            figure {
                pre { code { "$ " (recording.command) "\n" (recording.output) } }
                @if let Some(caption) = &props.caption {
                    figcaption { (caption) }
                }
            }
        })
    });

    directives
}

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

/// The landing page, which is a composition rather than prose.
///
/// This is what `[[page]]` in `site.toml` is for: a page whose body is Rust.
/// It is declared there, rendered here, and from then on it is a page like
/// any other -- in the navigation, in the sitemap, link-checked, and in the
/// `Route` enum.
pub fn landing(config: &Config) -> Markup {
    html! {
        h1 { "Typed routes" }
        p {
            "This page has no Markdown file. Its body is the function you are "
            "reading, and it is still a route the compiler knows about."
        }
        (sitemap(config))
    }
}

fn main() -> ExitCode {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let config = match Config::load(dir) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("crabbucket-example-site: {err}");
            return ExitCode::FAILURE;
        }
    };

    let options = Options::new()
        .page("", landing(&config).into_string())
        .directives(directives());

    match crabbucket::build_with(dir, &Plain, options) {
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
        {
            let mut config = Config::blank();
            config.title = "t".into();
            config.description = String::new();
            config.url = None;
            config.base = "/repo/".into();
            config
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
