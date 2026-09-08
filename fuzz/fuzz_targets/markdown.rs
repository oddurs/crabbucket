// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Markdown, highlighting and the directive scanner, on arbitrary bytes.
// Rendering may fail; it may not panic.  A build that fails names the file and
// the line, and a build that panics names a line of somebody else's crate.

#![no_main]

use crabbucket::Config;
use crabbucket::directive::{Context, Data, Directives};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|source: &str| {
    let config = Config::blank();
    let data = Data::default();
    let context = Context::new(&config, &data);

    let _ = crabbucket::markdown::render(source, &Directives::new(), &context);
});
