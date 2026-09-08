// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A directive's attributes are a TOML inline table written by hand in the
// middle of a Markdown file, and the scanner that finds them is line-based so
// that a directive inside a code fence is left alone.  Both are worth fuzzing:
// the fence tracking is the part with state in it.

#![no_main]

use crabbucket::Config;
use crabbucket::directive::{Context, Data, Directives};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Note {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    level: Option<u32>,
}

fuzz_target!(|source: &str| {
    let config = Config::blank();
    let data = Data::default();
    let context = Context::new(&config, &data);

    let mut directives = Directives::new();
    directives.add("note", |_props: Note, body| body);

    let _ = crabbucket::markdown::render(source, &directives, &context);
});
