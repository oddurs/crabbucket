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

//! `crab` -- build and check crabbucket sites.
//!
//! There is deliberately no `crab dev`.  turborust already supervises the
//! build, derives its watch globs from the crate closure, serves `dist/` with
//! live reload and shows rustc diagnostics in a browser overlay; a second,
//! worse version of that living here would be indefensible.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crabbucket_ui::Standard;

const PACKAGE: &str = "crab";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        [] | ["--help"] | ["-h"] | ["help"] => {
            print!("{}", help());
            ExitCode::SUCCESS
        }
        ["--version"] => {
            print!("{}", version());
            ExitCode::SUCCESS
        }
        ["build"] => build(Path::new(".")),
        ["build", dir] => build(Path::new(dir)),
        ["serve", ..] | ["dev", ..] => {
            eprintln!("{PACKAGE}: there is no built-in dev server; run `turborust up'.");
            eprintln!("See <https://github.com/oddurs/turborust>.");
            ExitCode::from(2)
        }
        [unknown, ..] => {
            eprintln!("{PACKAGE}: unrecognized argument '{unknown}'");
            eprintln!("Try '{PACKAGE} --help' for more information.");
            ExitCode::from(2)
        }
    }
}

/// Builds a site and reports what was written.
fn build(site_dir: &Path) -> ExitCode {
    match crabbucket::build(site_dir, &Standard) {
        Ok(report) => {
            let pages = report.routes.len();
            let plural = if pages == 1 { "page" } else { "pages" };
            println!(
                "{PACKAGE}: {pages} {plural}, {} link{} checked -> {}",
                report.links,
                if report.links == 1 { "" } else { "s" },
                display(&report.out_dir)
            );
            if report.drafts > 0 {
                println!("{PACKAGE}: {} draft(s) skipped", report.drafts);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{PACKAGE}: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Renders a path for a diagnostic, keeping it relative where that is shorter.
fn display(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.strip_prefix(&cwd)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// The `--help` text, in the shape the GNU coding standards ask for.
fn help() -> String {
    format!(
        "Usage: {PACKAGE} COMMAND [DIRECTORY]\n\
         Build a crabbucket site.\n\
         \n\
         Commands:\n\
         \x20 build [DIRECTORY]  build the site in DIRECTORY (default: .) to dist/\n\
         \n\
         \x20     --help         display this help and exit\n\
         \x20     --version      output version information and exit\n\
         \n\
         There is no dev server: run `turborust up' instead.\n\
         \n\
         Report bugs to: <https://github.com/oddurs/crabbucket/issues>\n\
         crabbucket home page: <https://github.com/oddurs/crabbucket>\n"
    )
}

/// The `--version` text, in the shape the GNU coding standards ask for.
fn version() -> String {
    format!(
        "{PACKAGE} (crabbucket) {VERSION}\n\
         Copyright (C) 2026 Oddur Sigurdsson\n\
         License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\n\
         This is free software: you are free to change and redistribute it.\n\
         There is NO WARRANTY, to the extent permitted by law.\n\
         \n\
         Written by Oddur Sigurdsson.\n"
    )
}
