// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every Rust block in the book is this crate's source.
//!
//! A book with a snippet that no longer compiles is worse than no book,
//! because it is confidently wrong.  The usual defence is a checker that
//! extracts the snippets and tries to build them, which drifts the moment
//! somebody adds a snippet the checker cannot wrap.
//!
//! This goes the other way round.  The snippets *are* `crabbucket-book-samples`
//! -- a design system called Ferrite and the site crate that uses it, both of
//! which the workspace builds and tests -- and this asserts that what the book
//! prints is what the crate contains, character for character.  So "does the
//! book still compile?" is answered by `cargo build`, and "does the book still
//! say what the code says?" is answered here.
//!
//! A block that is deliberately not real code -- an error message, a sketch of
//! an API that does not exist yet -- says so on the fence:
//!
//!     ```rust sketch
//!
//! which is a flag the Markdown renderer ignores and this test skips.

use std::fs;
use std::path::{Path, PathBuf};

/// The chapters, in the order the book reads.
fn chapters() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../site/content/book");
    let mut found: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("{}: {err}", dir.display()))
        .map(|entry| entry.expect("unreadable").path())
        .filter(|path| path.extension().is_some_and(|e| e == "md"))
        .map(|path| {
            let name = path
                .file_name()
                .expect("no name")
                .to_string_lossy()
                .into_owned();
            let body = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("{}: {err}", path.display()))
                .replace("\r\n", "\n");

            (name, body)
        })
        .collect();

    found.sort();
    assert!(!found.is_empty(), "{}: no chapters", dir.display());
    found
}

/// Every `.rs` file in this crate, concatenated, with line endings normalised.
///
/// A checkout on Windows has CRLF and the book does not, so a block written
/// with `\n` would otherwise match nothing at all.
fn source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut all = fs::read_to_string(root.join("build.rs")).unwrap_or_default();

    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
            let path = entry.expect("unreadable").path();

            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                all.push('\n');
                all.push_str(&fs::read_to_string(&path).unwrap_or_default());
            }
        }
    }

    all.replace("\r\n", "\n")
}

/// One fenced block: which chapter it is in, the line it opens on, the info
/// string, and the code.
struct Block {
    chapter: String,
    line: usize,
    info: String,
    code: String,
}

fn blocks(chapter: &str, body: &str) -> Vec<Block> {
    let mut found = Vec::new();
    let mut open: Option<(usize, String, String)> = None;

    for (index, line) in body.lines().enumerate() {
        match (&mut open, line.strip_prefix("```")) {
            // A fence inside a fence is content, not a fence.
            (Some((_, _, code)), None) => {
                code.push_str(line);
                code.push('\n');
            }
            (Some(_), Some(_)) => {
                let (line, info, code) = open.take().expect("open");
                found.push(Block {
                    chapter: chapter.to_string(),
                    line,
                    info,
                    code,
                });
            }
            (None, Some(info)) => open = Some((index + 1, info.trim().to_string(), String::new())),
            (None, None) => {}
        }
    }

    assert!(open.is_none(), "{chapter}: an unterminated fence");
    found
}

#[test]
fn every_rust_block_in_the_book_is_in_the_crate() {
    let source = source();
    let mut checked = 0;

    for (chapter, body) in chapters() {
        for block in blocks(&chapter, &body) {
            let mut words = block.info.split_whitespace();

            if words.next() != Some("rust") || words.any(|flag| flag == "sketch") {
                continue;
            }

            assert!(
                source.contains(block.code.trim_end()),
                "{}:{}: this block is not in crabbucket-book-samples, so nothing \
                 compiles it.  Either put it there, or mark the fence \
                 ```rust sketch if it is deliberately not real code.\n\n{}",
                block.chapter,
                block.line,
                block.code
            );

            checked += 1;
        }
    }

    // A test that silently checks nothing is the failure mode here: a renamed
    // directory, a changed fence, and this passes forever.
    assert!(checked >= 8, "only {checked} Rust blocks were checked");
}

#[test]
fn the_chapters_are_numbered_and_in_order() {
    let chapters = chapters();
    let mut orders = Vec::new();

    for (name, body) in &chapters {
        let order = body
            .lines()
            .find_map(|line| line.strip_prefix("order = "))
            .unwrap_or_else(|| panic!("{name}: no `order' in the frontmatter"))
            .trim()
            .parse::<usize>()
            .unwrap_or_else(|err| panic!("{name}: `order' is not a number: {err}"));

        orders.push((order, name.clone()));
    }

    orders.sort();
    let expected: Vec<usize> = (0..chapters.len()).collect();
    let actual: Vec<usize> = orders.iter().map(|(order, _)| *order).collect();

    assert_eq!(
        actual, expected,
        "the chapters skip or repeat an order: {orders:?}"
    );
}
