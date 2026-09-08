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

//! crabbucket -- a static site framework where the site is a typed value.
//!
//! The organising idea is stated in `doc/DESIGN`: the failures that break a
//! small documentation site are almost all string failures -- a dead link, a
//! page with no title, a renamed design token, a base path that is right in
//! development and wrong on GitHub Pages.  Every one of those is representable
//! as a type, so every one of them can fail the build instead of the site.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod content;
pub mod error;
pub mod links;
pub mod markdown;
pub mod site;
pub mod theme;
pub mod url;

pub use config::Config;
pub use content::{Collection, Entry};
pub use error::{Error, Result};
pub use site::{Report, build};
pub use theme::{NavItem, Page, PageMeta, PageRef, SiteIndex, Theme};
pub use url::Url;

/// Re-exported so that a site crate needs only one dependency to write
/// components.
pub use maud;
