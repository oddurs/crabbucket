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
//!
//! Arguments are parsed by hand.  It is forty lines, it follows the GNU
//! conventions the rest of the project follows, and a derive-macro dependency
//! to save those forty lines would be a bad trade for a tool this size.

#![forbid(unsafe_code)]

mod scaffold;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crabbucket::site::Options;
use crabbucket_ui::Standard;

const PACKAGE: &str = "crab";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A usage error: the command line was wrong, not the site.
const USAGE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match parse(&args) {
        Ok(Command::Help) => {
            print!("{}", help());
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            print!("{}", version());
            ExitCode::SUCCESS
        }
        Ok(Command::Build { dir, options }) => build(&dir, &options),
        Ok(Command::New(request)) => new(&request),
        Ok(Command::NoDevServer) => {
            eprintln!("{PACKAGE}: there is no built-in dev server; run `turborust up'.");
            eprintln!("See <https://github.com/oddurs/turborust>.");
            ExitCode::from(USAGE)
        }
        Err(message) => {
            eprintln!("{PACKAGE}: {message}");
            eprintln!("Try '{PACKAGE} --help' for more information.");
            ExitCode::from(USAGE)
        }
    }
}

/// What the command line asked for.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    Build { dir: PathBuf, options: Options },
    New(scaffold::Request),
    NoDevServer,
}

/// Parses the command line.
///
/// A bare `--` ends option processing, so a directory that begins with a dash
/// is still reachable.
fn parse(args: &[String]) -> Result<Command, String> {
    let mut args = args.iter().map(String::as_str);

    let Some(first) = args.next() else {
        return Ok(Command::Help);
    };

    match first {
        "--help" | "-h" | "help" => Ok(Command::Help),
        "--version" => Ok(Command::Version),
        "dev" | "serve" => Ok(Command::NoDevServer),
        "build" => parse_build(args),
        "new" => parse_new(args),
        other if other.starts_with('-') => Err(format!("unrecognized option '{other}'")),
        other => Err(format!("unrecognized command '{other}'")),
    }
}

fn parse_build<'a>(args: impl Iterator<Item = &'a str>) -> Result<Command, String> {
    let mut dir: Option<PathBuf> = None;
    let mut options = Options::default();
    let mut operands_only = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg {
            "--" if !operands_only => operands_only = true,
            "--out" if !operands_only => options.out_dir = Some(value(&mut args, "--out")?.into()),
            "--base" if !operands_only => options.base = Some(value(&mut args, "--base")?.into()),
            other if !operands_only && other.starts_with('-') => {
                return Err(format!("unrecognized option '{other}'"));
            }
            other if dir.is_none() => dir = Some(other.into()),
            other => return Err(format!("unexpected argument '{other}'")),
        }
    }

    Ok(Command::Build {
        dir: dir.unwrap_or_else(|| ".".into()),
        options,
    })
}

fn parse_new<'a>(args: impl Iterator<Item = &'a str>) -> Result<Command, String> {
    let mut request = scaffold::Request::default();
    let mut operands_only = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg {
            "--" if !operands_only => operands_only = true,
            "--base" if !operands_only => request.base = Some(value(&mut args, "--base")?.into()),
            "--router" if !operands_only => request.router = true,
            "--force" if !operands_only => request.force = true,
            other if !operands_only && other.starts_with('-') => {
                return Err(format!("unrecognized option '{other}'"));
            }
            other if request.dir.as_os_str().is_empty() => request.dir = other.into(),
            other => return Err(format!("unexpected argument '{other}'")),
        }
    }

    if request.dir.as_os_str().is_empty() {
        return Err("new: a directory name is required".to_string());
    }

    Ok(Command::New(request))
}

/// Takes the value belonging to an option that requires one.
fn value<'a>(
    args: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    option: &str,
) -> Result<&'a str, String> {
    args.next()
        .ok_or_else(|| format!("option '{option}' requires an argument"))
}

/// Builds a site and reports what was written.
fn build(site_dir: &Path, options: &Options) -> ExitCode {
    match crabbucket::site::build_with(site_dir, &Standard, options) {
        Ok(report) => {
            let pages = report.routes.len();
            println!(
                "{PACKAGE}: {pages} {}, {} {} checked -> {}",
                plural(pages, "page", "pages"),
                report.links,
                plural(report.links, "link", "links"),
                display(&report.out_dir)
            );
            for warning in &report.warnings {
                eprintln!("{PACKAGE}: warning: {warning}");
            }
            if report.drafts > 0 {
                println!("{PACKAGE}: {} draft(s) skipped", report.drafts);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{PACKAGE}: {}", err.render(std::io::stderr().is_terminal()));
            ExitCode::FAILURE
        }
    }
}

/// Scaffolds a site.
fn new(request: &scaffold::Request) -> ExitCode {
    match scaffold::write(request) {
        Ok(written) => {
            println!("{PACKAGE}: {} files -> {}", written, display(&request.dir));
            println!("\nNext:");
            println!("  cd {} && crab build", request.dir.display());
            println!("  turborust up");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{PACKAGE}: {message}");
            ExitCode::FAILURE
        }
    }
}

fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
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
        "Usage: {PACKAGE} COMMAND [OPTION]... [DIRECTORY]\n\
         Build a crabbucket site.\n\
         \n\
         Commands:\n\
         \x20 build [DIRECTORY]  build the site in DIRECTORY (default: .)\n\
         \x20 new DIRECTORY      scaffold a new site in DIRECTORY\n\
         \n\
         Options for build:\n\
         \x20     --out PATH     write the site to PATH instead of dist/\n\
         \x20     --base PATH    serve from PATH instead of the configured base\n\
         \n\
         Options for new:\n\
         \x20     --base PATH    the path the site will be served from\n\
         \x20     --router       ship the client-side router\n\
         \x20     --force        scaffold into a directory that is not empty\n\
         \n\
         \x20     --help         display this help and exit\n\
         \x20     --version      output version information and exit\n\
         \n\
         Exit status is 0 on success, 1 if the site is wrong, 2 if the command\n\
         line is.\n\
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Command, parse};

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn no_arguments_is_help() {
        assert_eq!(parse(&args(&[])).unwrap(), Command::Help);
    }

    #[test]
    fn build_defaults_to_the_current_directory() {
        let Command::Build { dir, options } = parse(&args(&["build"])).unwrap() else {
            panic!("not a build");
        };
        assert_eq!(dir, PathBuf::from("."));
        assert!(options.out_dir.is_none() && options.base.is_none());
    }

    #[test]
    fn build_takes_a_directory_and_both_overrides() {
        let Command::Build { dir, options } = parse(&args(&[
            "build", "site", "--out", "/tmp/x", "--base", "/repo/",
        ]))
        .unwrap() else {
            panic!("not a build");
        };
        assert_eq!(dir, PathBuf::from("site"));
        assert_eq!(options.out_dir, Some(PathBuf::from("/tmp/x")));
        assert_eq!(options.base.as_deref(), Some("/repo/"));
    }

    #[test]
    fn a_double_dash_ends_option_processing() {
        let Command::Build { dir, .. } = parse(&args(&["build", "--", "--odd"])).unwrap() else {
            panic!("not a build");
        };
        assert_eq!(dir, PathBuf::from("--odd"));
    }

    #[test]
    fn an_option_without_its_argument_is_a_usage_error() {
        assert!(parse(&args(&["build", "--out"])).is_err());
    }

    #[test]
    fn unknown_options_and_commands_are_rejected() {
        assert!(parse(&args(&["build", "--nope"])).is_err());
        assert!(parse(&args(&["frobnicate"])).is_err());
        assert!(parse(&args(&["--nope"])).is_err());
    }

    #[test]
    fn dev_and_serve_both_point_at_turborust() {
        assert_eq!(parse(&args(&["dev"])).unwrap(), Command::NoDevServer);
        assert_eq!(parse(&args(&["serve"])).unwrap(), Command::NoDevServer);
    }

    #[test]
    fn new_requires_a_directory() {
        assert!(parse(&args(&["new"])).is_err());
        assert!(parse(&args(&["new", "site"])).is_ok());
    }

    /// Every long option, from either the help text or the man page.
    fn options(text: &str) -> std::collections::BTreeSet<String> {
        let mut found = std::collections::BTreeSet::new();
        let mut rest = text;

        while let Some(at) = rest.find("--") {
            rest = &rest[at..];
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .unwrap_or(rest.len());
            let option = rest[..end].trim_end_matches(['-', '.', ',']);

            // `\-\-out` in roff, and a bare `--`, are not options.
            if option.len() > 2 && !option.contains("\\") {
                found.insert(option.to_string());
            }

            rest = &rest[end.max(2)..];
        }

        found
    }

    #[test]
    fn the_man_page_documents_exactly_the_options_help_does() {
        let page = include_str!("../../../doc/crab.1").replace("\\-", "-");
        let documented = options(&page);
        let helped = options(&super::help());

        assert!(
            helped.len() >= 6,
            "the option scanner found nothing: {helped:?}"
        );

        assert_eq!(
            helped,
            documented,
            "--help and doc/crab.1 disagree; only in --help: {:?}; only in the man page: {:?}",
            helped.difference(&documented).collect::<Vec<_>>(),
            documented.difference(&helped).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn help_documents_every_option_the_parser_accepts() {
        let text = super::help();
        for option in [
            "--out",
            "--base",
            "--router",
            "--force",
            "--help",
            "--version",
        ] {
            assert!(text.contains(option), "{option} is not in --help");
        }
    }
}
