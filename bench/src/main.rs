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
//! Generates a synthetic site and times building it.
//!
//! `doc/DESIGN` opens by conceding that speed is not the point -- "Hugo builds
//! it in 30ms and nobody is unhappy".  That is an honest position and it stays
//! honest only if the numbers are known, so this makes them known.
//!
//! The same content is emitted for crabbucket, Hugo and Zola.  It is not
//! identical work -- each does things the others do not -- but it is the same
//! pages, the same prose, and the same code blocks, which is as close as a
//! comparison between three different tools gets.
//!
//!     cargo run -p crabbucket-bench -- generate 500 /tmp/bench
//!     cargo run -p crabbucket-bench -- time /tmp/bench
//!     cargo run -p crabbucket-bench -- budget /tmp/bench 600
//!
//! `budget` is the one CI runs: it builds the same synthetic site and exits
//! non-zero if the best of three runs is slower than the ceiling given.  The
//! ceiling is deliberately loose -- CI hardware is shared and noisy, and a
//! budget that fails on noise is a budget everybody learns to ignore.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Instant;

use crabbucket::Timings;

/// Words of filler, so a page is prose rather than a title.
const LOREM: &str = "\
A site is a typed value. Every static site generator in wide use is a string \
pipeline: templates render strings, frontmatter is a loose map, links are \
strings, and the base path is a string. Every one of those is a place a small \
site breaks in a way you only find in production. The advantage of a typed \
build is not speed but that an entire class of bug stops being a runtime \
surprise.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        ["generate", pages, dir] => match pages.parse() {
            Ok(pages) => generate(pages, Path::new(dir)),
            Err(_) => usage("the page count is not a number"),
        },
        ["time", dir] => time(Path::new(dir)),
        ["budget", dir, ms] => match ms.parse() {
            Ok(ms) => budget(Path::new(dir), ms),
            Err(_) => usage("the budget is not a number of milliseconds"),
        },
        _ => usage("usage: crabbucket-bench generate PAGES DIR | time DIR | budget DIR MS"),
    }
}

fn usage(message: &str) -> ExitCode {
    eprintln!("crabbucket-bench: {message}");
    ExitCode::from(2)
}

/// Writes the same site three ways.
fn generate(pages: usize, dir: &Path) -> ExitCode {
    let _ = fs::remove_dir_all(dir);

    if let Err(err) = write_all(pages, dir) {
        eprintln!("crabbucket-bench: {err}");
        return ExitCode::FAILURE;
    }

    println!("crabbucket-bench: {pages} pages -> {}", dir.display());
    ExitCode::SUCCESS
}

fn write_all(pages: usize, dir: &Path) -> std::io::Result<()> {
    // A deep tree rather than a flat one, because a real docs site is nested
    // and route derivation is per-component.
    let section = |page: usize| format!("section-{}", page % 20);

    for generator in ["crabbucket", "hugo", "zola"] {
        let root = dir.join(generator);
        fs::create_dir_all(&root)?;

        for page in 0..pages {
            let body = page_body(page);
            let path = match generator {
                "crabbucket" => root
                    .join("content")
                    .join(section(page))
                    .join(format!("page-{page}.md")),
                "hugo" => root
                    .join("content")
                    .join(section(page))
                    .join(format!("page-{page}.md")),
                _ => root
                    .join("content")
                    .join(section(page))
                    .join(format!("page-{page}.md")),
            };

            fs::create_dir_all(path.parent().expect("no parent"))?;
            fs::write(path, body)?;
        }

        // Every generator needs a home page, and the sections need indexes or
        // Hugo and Zola render fewer pages than crabbucket does.
        let home = match generator {
            "crabbucket" => root.join("content/index.md"),
            _ => root.join("content/_index.md"),
        };
        fs::write(home, page_body(0))?;

        // Section indexes, so all three write the same number of routes: a
        // comparison where one generator renders twenty fewer pages is not one.
        for index in 0..20 {
            let name = if generator == "crabbucket" {
                "index.md"
            } else {
                "_index.md"
            };
            fs::write(
                root.join("content")
                    .join(format!("section-{index}"))
                    .join(name),
                page_body(index),
            )?;
        }

        write_config(generator, &root, pages)?;
    }

    Ok(())
}

/// One page: frontmatter, prose, a heading tree, a table and a code block.
fn page_body(page: usize) -> String {
    let mut out = String::new();

    // The frontmatter is deliberately the intersection of what the three
    // accept, so the byte-for-byte input is the same for all of them.
    let _ = write!(
        out,
        "+++\ntitle = \"Page {page}\"\ndescription = \"Page {page}.\"\n+++\n\n"
    );

    let _ = write!(out, "# Page {page}\n\n{LOREM}\n\n");

    for heading in 0..4 {
        let _ = write!(out, "## Heading {heading}\n\n{LOREM}\n\n");
        let _ = write!(
            out,
            "```rust\npub fn heading_{heading}(page: usize) -> String {{\n    format!(\"{{page}}\")\n}}\n```\n\n"
        );
    }

    let _ = write!(
        out,
        "| Column | Column |\n|---|---|\n| a | b |\n| c | d |\n\n"
    );
    out
}

fn write_config(generator: &str, root: &Path, _pages: usize) -> std::io::Result<()> {
    match generator {
        "crabbucket" => fs::write(
            root.join("site.toml"),
            "title = \"Benchmark\"\ndescription = \"A synthetic site.\"\nbase = \"/\"\n",
        ),
        "hugo" => {
            fs::create_dir_all(root.join("layouts/_default"))?;
            fs::write(
                root.join("layouts/_default/single.html"),
                "<!doctype html><html><head><title>{{ .Title }}</title></head>\
                 <body><main>{{ .Content }}</main></body></html>",
            )?;
            fs::write(
                root.join("layouts/_default/list.html"),
                "<!doctype html><html><head><title>{{ .Title }}</title></head>\
                 <body><main>{{ .Content }}</main></body></html>",
            )?;
            fs::write(
                root.join("hugo.toml"),
                "baseURL = \"/\"\ntitle = \"Benchmark\"\n",
            )
        }
        _ => {
            fs::create_dir_all(root.join("templates"))?;
            fs::write(
                root.join("templates/page.html"),
                "<!doctype html><html><head><title>{{ page.title }}</title></head>\
                 <body><main>{{ page.content | safe }}</main></body></html>",
            )?;
            fs::write(
                root.join("templates/section.html"),
                "<!doctype html><html><head><title>{{ section.title }}</title></head>\
                 <body><main>{{ section.content | safe }}</main></body></html>",
            )?;
            fs::write(
                root.join("templates/index.html"),
                "<!doctype html><html><head><title>Benchmark</title></head><body></body></html>",
            )?;
            fs::write(
                root.join("config.toml"),
                "base_url = \"https://example.com\"\ntitle = \"Benchmark\"\n\n\
                 [markdown.highlighting]\ntheme = \"ayu-dark\"\n",
            )
        }
    }
}

/// Times each generator, three runs, reporting the fastest.
fn time(dir: &Path) -> ExitCode {
    println!("{:<12} {:>9}  notes", "generator", "best");
    println!("{:-<12} {:->9}  {:-<28}", "", "", "");

    let ours = crabbucket(dir);

    report(
        "crabbucket",
        ours.map(|(ms, _)| ms),
        "highlighting, link checking",
    );

    for (name, program, args, note) in [
        (
            "hugo",
            "hugo",
            vec!["--quiet", "--destination", "public"],
            "highlighting",
        ),
        ("zola", "zola", vec!["build"], "highlighting"),
    ] {
        if Command::new(program).arg("--version").output().is_err() {
            println!("{name:<12} {:>9}  not installed", "-");
            continue;
        }

        let root = dir.join(name);
        let timing = best(3, || {
            Command::new(program)
                .args(&args)
                .current_dir(&root)
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|_| 0)
        });

        report(name, timing, note);
    }

    // Only crabbucket's passes are known, because only crabbucket is being
    // asked to account for itself.  Hugo and Zola are here as a yardstick, not
    // as subjects.
    if let Some((total, phases)) = ours {
        println!("\ncrabbucket, by pass:");
        for (name, phase) in [
            ("read and parse", phases.read),
            ("render", phases.render),
            ("check links", phases.check),
            ("write", phases.write),
        ] {
            let share = phase.as_secs_f64() / phases.total().as_secs_f64() * 100.0;
            println!("  {name:<16} {:>5}ms  {share:>4.0}%", phase.as_millis());
        }
        println!("                   {total:>5}ms   wall");
    }

    ExitCode::SUCCESS
}

/// Builds the crabbucket site three times, keeping the fastest and its passes.
fn crabbucket(dir: &Path) -> Option<(u128, Timings)> {
    let site = dir.join("crabbucket");
    let mut fastest: Option<(u128, Timings)> = None;

    for _ in 0..3 {
        let started = Instant::now();
        let report = crabbucket::build(&site, &crabbucket_ui::Standard).ok()?;
        let elapsed = started.elapsed().as_millis();

        if fastest.is_none_or(|(best, _)| elapsed < best) {
            fastest = Some((elapsed, report.timings));
        }
    }

    fastest
}

/// Fails if the build is slower than the ceiling.  This is what CI runs.
fn budget(dir: &Path, ceiling: u128) -> ExitCode {
    let Some((ms, phases)) = crabbucket(dir) else {
        eprintln!("crabbucket-bench: the build failed");
        return ExitCode::FAILURE;
    };

    let passes = format!(
        "read {}ms, render {}ms, check {}ms, write {}ms",
        phases.read.as_millis(),
        phases.render.as_millis(),
        phases.check.as_millis(),
        phases.write.as_millis()
    );

    if ms > ceiling {
        eprintln!("crabbucket-bench: {ms}ms, over the {ceiling}ms budget ({passes})");
        return ExitCode::FAILURE;
    }

    println!("crabbucket-bench: {ms}ms, within the {ceiling}ms budget ({passes})");
    ExitCode::SUCCESS
}

/// The fastest of `runs` attempts, in milliseconds, or `None` if it failed.
fn best(runs: usize, mut once: impl FnMut() -> Option<usize>) -> Option<u128> {
    let mut fastest = None;

    for _ in 0..runs {
        let started = Instant::now();
        once()?;
        let elapsed = started.elapsed().as_millis();
        fastest = Some(fastest.map_or(elapsed, |best: u128| best.min(elapsed)));
    }

    fastest
}

fn report(name: &str, timing: Option<u128>, note: &str) {
    match timing {
        Some(ms) => println!("{name:<12} {:>8}ms  {note}", ms),
        None => println!("{name:<12} {:>9}  failed", "-"),
    }
}
