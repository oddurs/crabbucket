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

//! What comes out of a build, checked rather than assumed.
//!
//! Two claims are made in writing and nothing held either of them.
//!
//! `doc/DESIGN' and the deploying page both say a build produces the same
//! output on every machine.  Nothing built twice and compared.
//!
//! "The build either fails, or the site is correct" is the whole product, and
//! nothing checked that the HTML it emits has balanced tags or that the feeds
//! it emits are XML at all.  A feed that does not parse is not a feed; it is a
//! file with a `.xml' extension, and the reader that finds out is somebody's.
//!
//! This is in `crabbucket-ui' rather than in `crabbucket' because it needs a
//! real design system -- one with a router, a search client, social cards and
//! feeds -- and the dependency runs this way round.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crabbucket::{Options, Report};
use crabbucket_ui::Standard;

/// Writes the fixture site, in a directory of its own.
///
/// Its own because these tests run in parallel and each begins by removing
/// what it is about to write.
fn site(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    let write = |path: &str, body: &str| {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().expect("no parent")).expect("mkdir");
        fs::write(path, body).expect("write");
    };

    // `url' turns on everything that needs an absolute address: the sitemap,
    // robots.txt, the feeds and the social cards.  All of them are output this
    // file is about, so all of them are on.
    write(
        "site.toml",
        "title = \"Fixture\"\n\
         description = \"A site with one of everything.\"\n\
         base = \"/fixture/\"\n\
         url = \"https://example.com/fixture/\"\n\
         search = true\n\
         router = true\n\
         \n\
         [[feed]]\n\
         collection = \"notes\"\n\
         title = \"Notes\"\n\
         limit = 10\n",
    );

    // No *raw* HTML in the prose: CommonMark passes it through, deliberately
    // and correctly, so `<angles>' written bare in a paragraph is an unclosed
    // tag by the author's own instruction rather than a fault of the build.
    // The angles below are in a code span, where they must come out escaped.
    write(
        "content/index.md",
        "+++\ntitle = \"Home\"\nnav_order = 1\n+++\n\n\
         # Home\n\n\
         Prose with a [link](/notes/) and an ampersand & a `code span`.\n\n\
         :::callout{kind = \"warn\", title = \"Careful\"}\n\
         Attributes with \"quotes\" in them, and `<angles>` in a code span.\n\
         :::\n",
    );

    write(
        "content/notes/index.md",
        "+++\ntitle = \"Notes\"\nlayout = \"docs\"\ndate = 2026-01-02\nnav_order = 2\n+++\n\n\
         # Notes\n",
    );

    write(
        "content/notes/first.md",
        "+++\ntitle = \"First & last\"\nlayout = \"docs\"\ndate = 2026-01-03\n+++\n\n\
         # First & last\n\n\
         ```rust\nfn main() { println!(\"<hello>\"); }\n```\n\n\
         | a | b |\n|---|---|\n| c | d |\n\n\
         Back [home](~/).\n",
    );

    write(
        "content/404.md",
        "+++\ntitle = \"Not found\"\n+++\n\n# Not found\n\nTry [home](~/).\n",
    );

    dir
}

fn build(name: &str) -> (Report, PathBuf) {
    let dir = site(name);
    let out = dir.join("dist");
    let report = crabbucket::build_with(&dir, &Standard, Options::new().out_dir(out.clone()))
        .unwrap_or_else(|err| panic!("the fixture should build, but: {err}"));

    (report, out)
}

/// Every file under `dir`, by its path relative to it.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).unwrap_or_else(|err| panic!("{current:?}: {err}")) {
            let path = entry.expect("unreadable").path();

            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(dir)
                    .expect("not under dir")
                    .to_string_lossy()
                    .replace('\\', "/");

                found.insert(relative, fs::read(&path).expect("unreadable"));
            }
        }
    }

    found
}

// ------------------------------------------------------------ determinism

#[test]
fn the_same_site_built_twice_is_the_same_bytes() {
    // Two directories rather than two builds in one, so that neither the
    // social-card cache nor anything else can carry state from the first into
    // the second.  This is the claim doc/DESIGN makes, and a deploy that
    // diffs its output is the thing that would otherwise find out.
    let (_, first) = build("determinism-a");
    let (_, second) = build("determinism-b");

    let left = tree(&first);
    let right = tree(&second);

    let names: Vec<&String> = left.keys().collect();
    let others: Vec<&String> = right.keys().collect();
    assert_eq!(names, others, "the two builds wrote different files");

    for (name, bytes) in &left {
        assert_eq!(
            bytes, &right[name],
            "{name} differs between two builds of the same site"
        );
    }

    assert!(
        left.len() > 10,
        "only {} files; is this a build?",
        left.len()
    );
}

// ---------------------------------------------------------------- validity

/// The HTML elements that have no closing tag, so a balance check does not
/// wait for one.
const VOID: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Every unbalanced tag in a document, as `(tag, why)`.
///
/// Not an HTML parser: a real one is a large dependency to hold one property,
/// and the failure this is looking for is coarse.  A design system builds its
/// markup with `maud', which cannot emit an unclosed tag, and the prose comes
/// from a Markdown renderer that cannot either -- so anything unbalanced here
/// came from the seam between them, which is exactly where it would.
fn unbalanced(html: &str) -> Vec<(String, String)> {
    let mut faults = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut rest = html;

    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];

        // Comments and the doctype are not elements.
        if rest.starts_with('!') {
            continue;
        }

        let closing = rest.starts_with('/');
        let body = if closing { &rest[1..] } else { rest };

        let Some(end) = body.find('>') else {
            faults.push((String::from("<"), String::from("a tag with no `>'")));
            break;
        };

        let inside = &body[..end];
        let self_closing = inside.ends_with('/');
        let name: String = inside
            .split([' ', '\t', '\n', '/', '>'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();

        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            continue;
        }

        if closing {
            match open.pop() {
                Some(last) if last == name => {}
                Some(last) => {
                    faults.push((name.clone(), format!("closed while `{last}' was open")))
                }
                None => faults.push((name.clone(), String::from("closed but never opened"))),
            }
        } else if !self_closing && !VOID.contains(&name.as_str()) {
            open.push(name);
        }
    }

    for name in open {
        faults.push((name, String::from("opened and never closed")));
    }

    faults
}

#[test]
fn the_balance_checker_notices_the_things_it_is_for() {
    // A checker nobody has seen fail is a checker that returns an empty vector.
    assert!(unbalanced("<p>fine</p><br><img src=x>").is_empty());
    assert!(unbalanced("<div><p>nested</p></div>").is_empty());

    assert_eq!(unbalanced("<p>never closed").len(), 1);
    assert_eq!(unbalanced("</p>").len(), 1);
    assert_eq!(unbalanced("<div><p></div></p>").len(), 2);
}

#[test]
fn every_page_the_build_writes_has_balanced_tags() {
    let (_, out) = build("balance");

    for (name, bytes) in tree(&out) {
        if !name.ends_with(".html") {
            continue;
        }

        let html = String::from_utf8(bytes).unwrap_or_else(|err| panic!("{name}: {err}"));
        let faults = unbalanced(&html);

        assert!(faults.is_empty(), "{name}: {faults:?}");
    }
}

#[test]
fn the_sitemap_and_both_feeds_are_well_formed_xml() {
    let (_, out) = build("xml");

    for name in ["sitemap.xml", "notes/feed.xml", "notes/atom.xml"] {
        let path = out.join(name);
        let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{name}: {err}"));

        let mut reader = quick_xml::Reader::from_str(&text);
        reader.config_mut().check_end_names = true;

        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => buffer.clear(),
                Err(err) => panic!("{name} is not well-formed XML: {err}\n\n{text}"),
            }
        }
    }
}

#[test]
fn the_search_index_is_json_and_does_not_escape_its_own_script_tag() {
    let (_, out) = build("search-index");
    let index = fs::read_to_string(out.join("search.json")).expect("no index");

    assert!(index.starts_with('['), "not an array: {index}");
    assert!(index.trim_end().ends_with(']'), "not an array: {index}");

    // The fixture has `<hello>' inside a code fence and `<angles>' inside a
    // directive.  Neither may reach the index as a tag that closes something.
    assert!(
        !index.contains("</script"),
        "the index can close a script tag"
    );
    assert!(!index.contains("\n]"), "unexpected formatting: {index}");
}

// --------------------------------------------------------------- snapshots

// A design system's output is asserted elsewhere with `contains()', which
// answers "is the thing I thought of still there?" and never notices anything
// else moving.  These answer the other question: what changed?
//
// Reviewing a change:
//
//     cargo insta review
//
// A diff that is the change you meant is the point of the test.  Accepting one
// without reading it is the same as not having it.

/// One tag per line, so a snapshot diff points at what moved.
///
/// `maud' emits no whitespace between elements, which is right for a page and
/// useless for a diff: a fifteen-kilobyte single line reports every change as
/// "line 1 differs".  This is not pretty-printing -- no indentation, nothing
/// reordered -- only a line break where a reader would want one.
fn readable(html: &str) -> String {
    html.replace('<', "\n<").trim_start().to_string()
}

#[test]
fn a_rendered_page_is_what_it_was() {
    let (_, out) = build("snapshot-page");
    let html = fs::read_to_string(out.join("notes/first/index.html")).expect("no page");

    // The docs layout, a code fence, a table, a resolved `~/', the section
    // list and the neighbour links -- most of the design system on one page.
    insta::assert_snapshot!("standard_docs_page", readable(&html));
}

#[test]
fn the_stylesheet_is_what_it_was() {
    let (_, out) = build("snapshot-css");
    let css = fs::read_to_string(out.join("site.css")).expect("no stylesheet");

    // Generated from the tokens and from every `Style' the theme lists, so a
    // renamed token, a dropped component or a broken `&' expansion all show
    // up here as a diff rather than as a page that quietly loses its colours.
    insta::assert_snapshot!("standard_stylesheet", css);
}
