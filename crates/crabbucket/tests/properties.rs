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

//! The claims, rather than the examples.
//!
//! The rest of the suite is example-based: a fixture goes in, an assertion
//! looks at what came out.  That is the right first layer and it has found
//! real bugs.  What it cannot do is hold a sentence like "every spelling of a
//! base path normalises to the same thing", because there are four spellings
//! in the test and infinitely many in the world -- and the interesting one is
//! never among the four somebody thought of.
//!
//! Each test below is a sentence from the documentation, turned into something
//! a machine can try to break.  Where one fails, the input it failed on is
//! worth adding to the example tests as well: a property test tells you that
//! something is wrong, and a named example test stops it coming back.

use crabbucket::{Config, Style, StyleSheet, Url};
use proptest::prelude::*;

/// A config with a given base, since that is the only field these touch.
fn with_base(base: &str) -> Config {
    let mut config = Config::blank();
    config.set_base(base);
    config
}

/// Text that a person might plausibly type into `site.toml` or a link.
///
/// Deliberately not `.*`: unrestricted Unicode finds that `char::to_lowercase`
/// is not a bijection, which is true, known, and not what any of this is
/// about.  This is the alphabet of paths, plus the characters that break them.
fn pathish() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z0-9._~/ -]{0,24}").expect("bad regex")
}

proptest! {
    /// "Every spelling of a base path normalises to the same thing."
    ///
    /// The documented claim, and the one that decides whether a site works
    /// under `/repo/`.  Normalising twice must be normalising once, or the
    /// value depends on how many times it has been through the function.
    #[test]
    fn normalising_a_base_twice_is_normalising_it_once(base in pathish()) {
        let once = with_base(&base).base;
        let twice = with_base(&once).base;

        prop_assert_eq!(&once, &twice);
    }

    /// A base is always a path: it begins and ends with a slash, whatever
    /// went in, so nothing downstream has to decide whether to add one.
    #[test]
    fn a_base_is_always_bounded_by_slashes(base in pathish()) {
        let base = with_base(&base).base;

        prop_assert!(base.starts_with('/'), "{base:?}");
        prop_assert!(base.ends_with('/'), "{base:?}");
    }

    /// The four spellings in the example test are four points on a line.
    /// Any amount of surrounding whitespace and any number of slashes at
    /// either end are the same base.
    #[test]
    fn slashes_and_spaces_around_a_base_do_not_change_it(
        inner in "[a-z][a-z0-9-]{0,12}",
        before in "[/ ]{0,4}",
        after in "[/ ]{0,4}",
    ) {
        let plain = with_base(&inner).base;
        let dressed = with_base(&format!("{before}{inner}{after}")).base;

        prop_assert_eq!(plain, dressed);
    }

    /// A page URL starts with the base and ends in a slash, so every route is
    /// a directory and no host needs rewrite rules.
    #[test]
    fn a_page_url_starts_at_the_base_and_ends_in_a_slash(
        base in pathish(),
        path in pathish(),
    ) {
        let config = with_base(&base);
        let url = Url::new(&config, &path);

        prop_assert!(url.as_str().starts_with(&config.base), "{url} vs {}", config.base);
        prop_assert!(url.as_str().ends_with('/'), "{url}");
    }

    /// The bug this is really about: `/repo/` and a path of `/docs/` becoming
    /// `/repo//docs/`.  A doubled slash is a different URL to a browser and
    /// the same one to a person, which is the worst combination available.
    #[test]
    fn no_url_ever_doubles_a_slash(base in pathish(), path in pathish()) {
        let config = with_base(&base);

        prop_assert!(!Url::new(&config, &path).as_str().contains("//"));
        prop_assert!(!Url::asset(&config, &path).as_str().contains("//"));
    }

    /// An asset keeps its extension and gets no trailing slash, because it is
    /// a file rather than a route.  The one exception is an empty path, which
    /// is the base itself.
    #[test]
    fn an_asset_url_is_a_file_not_a_directory(
        base in pathish(),
        name in "[a-z][a-z0-9-]{0,10}\\.(css|js|png|xml)",
    ) {
        let config = with_base(&base);
        let url = Url::asset(&config, &name);

        prop_assert!(url.as_str().starts_with(&config.base));
        prop_assert!(!url.as_str().ends_with('/'), "{url}");
        prop_assert!(url.as_str().ends_with(&name), "{url}");
    }
}

proptest! {
    /// `&` is the component's class root, and a stylesheet that still contains
    /// one is a stylesheet with a literal `.&` selector in it -- which matches
    /// nothing, silently, which is how it would ship.
    #[test]
    fn expanding_a_style_leaves_no_ampersand(
        namespace in "[a-z]{1,4}",
        name in "[a-z][a-z-]{0,10}",
        rules in "(\\.&(__[a-z]{1,6})?(--[a-z]{1,6})? \\{ color: red; \\}\n?){0,6}",
    ) {
        // `Style` holds `&'static str`, since a design system's styles are
        // compiled in.  A test that generates them has to leak, and does.
        let style = Style::new(
            Box::leak(namespace.into_boxed_str()),
            Box::leak(name.into_boxed_str()),
            Box::leak(rules.into_boxed_str()),
        );

        prop_assert!(!style.render().contains('&'), "{}", style.render());
    }

    /// Two design systems can both have a `page' and neither wins, which is
    /// only true while every rule carries its own namespace.
    #[test]
    fn every_expanded_selector_carries_its_namespace(
        namespace in "[a-z]{2,4}",
        name in "[a-z][a-z-]{0,8}",
    ) {
        let namespace: &'static str = Box::leak(namespace.into_boxed_str());
        let style = Style::new(
            namespace,
            Box::leak(name.into_boxed_str()),
            ".& { color: red; }\n.&__part { color: blue; }\n",
        );

        for selector in style.render().lines().filter(|line| line.starts_with('.')) {
            prop_assert!(
                selector.starts_with(&format!(".{namespace}-")),
                "{selector:?} is not namespaced"
            );
        }
    }

    /// A sheet is the styles it was given, and adding one twice adds it once:
    /// a design system that lists `PROSE' in two places should not ship the
    /// rules twice and let the second copy win by source order.
    #[test]
    fn a_stylesheet_holds_each_style_once(count in 1usize..6) {
        const STYLES: [Style; 5] = [
            Style::new("t", "a", ".& { color: red; }\n"),
            Style::new("t", "b", ".& { color: red; }\n"),
            Style::new("t", "c", ".& { color: red; }\n"),
            Style::new("t", "d", ".& { color: red; }\n"),
            Style::new("t", "e", ".& { color: red; }\n"),
        ];

        let chosen = &STYLES[..count];

        let mut once = StyleSheet::new();
        once.extend(chosen.iter().copied());

        let mut twice = StyleSheet::new();
        twice.extend(chosen.iter().copied());
        twice.extend(chosen.iter().copied());

        prop_assert_eq!(once.len(), count);
        prop_assert_eq!(twice.len(), count);
        prop_assert_eq!(once.render(), twice.render());
    }
}

// ------------------------------------------------- the parsers do not panic

// Everything below runs on whatever is in a content file.  None of it is
// allowed to panic: a build that fails names the file and the line, and a
// build that panics names a line of somebody else's crate.  The CRLF bug got
// through review on three platforms, so "somebody would have noticed" is not
// an argument that has held here.

use std::collections::{BTreeMap, BTreeSet};

use crabbucket::directive::{Context, Data, Directives};
use crabbucket::links::{self, Rendered};
use crabbucket::markdown;
use crabbucket::search;

/// Text with the characters that mean something to one of the parsers in it,
/// weighted so they actually turn up rather than appearing once in a thousand
/// runs of `[a-z]`.
fn contentish() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => "[a-z .]{1,8}",
            3 => Just("+++".to_string()),
            3 => Just(":::".to_string()),
            3 => Just("```".to_string()),
            2 => Just("\r\n".to_string()),
            2 => Just("\n".to_string()),
            2 => "[{}\\[\\]\"'=<>&#~/-]{1,3}",
            1 => Just("callout".to_string()),
            1 => Just("<script>".to_string()),
        ],
        0..40,
    )
    .prop_map(|parts| parts.concat())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    /// Markdown and the directive scanner run on arbitrary text.  Either may
    /// return an error naming the fault; neither may take the process down.
    #[test]
    fn rendering_arbitrary_text_never_panics(source in contentish()) {
        let config = Config::blank();
        let data = Data::default();
        let context = Context::new(&config, &data);
        let directives = Directives::new();

        let _ = markdown::render(&source, &directives, &context);
    }

    /// The search index is embedded in a page as JSON.  A document that can
    /// close its own string literal is a broken page at best.
    #[test]
    fn the_search_index_is_always_valid_json(source in contentish()) {
        let config = Config::blank();
        let data = Data::default();
        let context = Context::new(&config, &data);
        let directives = Directives::new();

        let Ok(body) = markdown::render(&source, &directives, &context) else {
            return Ok(());
        };

        let text = search::plain(&body.html);
        let index = search::index(&[search::Document {
            url: "/",
            title: &source,
            headings: &body.headings,
            text,
        }]);

        // Not a JSON parser, because that would be a dependency to hold one
        // property.  These are the two ways this has ever gone wrong.
        prop_assert!(index.starts_with('['), "{index}");
        prop_assert!(index.ends_with(']'), "{index}");
        prop_assert!(!index.contains("</script"), "{index}");
    }

    /// Link checking runs on rendered HTML, which is to say on whatever the
    /// Markdown made of whatever was in the file.
    #[test]
    fn checking_arbitrary_links_never_panics(html in contentish(), base in pathish()) {
        let config = with_base(&base);
        let anchors = BTreeSet::new();
        let pages = [Rendered {
            source: std::path::Path::new("content/index.md"),
            url: "/",
            anchors: &anchors,
            error_page: false,
            html: &html,
        }];

        let _ = links::check(&config, &pages, &BTreeMap::new(), &BTreeSet::new());
    }

    /// `~/' is resolved on the finished page, so it too runs on arbitrary
    /// HTML -- and it rewrites, which is the kind of thing that slices a
    /// string in the middle of a character.
    #[test]
    fn resolving_the_site_root_never_panics(html in contentish(), base in pathish()) {
        let _ = links::absolutize(&html, &with_base(&base));
    }
}

proptest! {
    /// A checkout on Windows has CRLF.  The same file, either way, is the same
    /// page -- which was not true until a bug found on a Windows runner made
    /// it true, and is exactly the sort of thing that quietly stops being so.
    #[test]
    fn line_endings_do_not_change_what_a_page_says(source in contentish()) {
        let config = Config::blank();
        let data = Data::default();
        let context = Context::new(&config, &data);
        let directives = Directives::new();

        let unix = source.replace("\r\n", "\n");
        let windows = unix.replace('\n', "\r\n");

        let render = |text: &str| {
            markdown::render(text, &directives, &context)
                .map(|body| body.html)
                .map_err(|fault| fault.message)
        };

        prop_assert_eq!(render(&unix), render(&windows));
    }
}
