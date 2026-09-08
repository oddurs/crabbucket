// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later

//! The consumer side: what each of the twelve sites contains.
//!
//! Chapter 6 of the book is about a house style used by a fleet of sites.  A
//! site that depends on a design system has to call the build itself, because
//! `crab build` only knows the design system it was compiled with -- so a site
//! in a fleet is a crate, and this is the whole of it.

use std::path::Path;
use std::process::ExitCode;

use crabbucket::{Options, Report, Result};

use crate::Ferrite;

/// The whole of a site's `main.rs`.
///
/// Twelve repositories contain this file and differ only in their content
/// directory.  Restyling all of them is a version bump in twelve
/// `Cargo.toml`s, which is the claim the design exists to make good on.
pub fn main() -> ExitCode {
    match crabbucket::build(Path::new("."), &Ferrite) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// The same build, told where it will be served from.
///
/// A project site on GitHub Pages lives under `/repository/`, and a preview
/// build does not.  The base path is a build input rather than something
/// written into the content, so the same content serves from both.
pub fn build_at(site: &Path, base: Option<String>) -> Result<Report> {
    crabbucket::build_with(site, &Ferrite, Options::new().maybe_base(base))
}
