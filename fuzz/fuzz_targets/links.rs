// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Link resolution and `~/' expansion, on arbitrary HTML.  Both scan a string
// and slice it, which is the shape of code that panics on a character boundary
// in the middle of something multi-byte.

#![no_main]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crabbucket::Config;
use crabbucket::links::{self, Rendered};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|html: &str| {
    let mut config = Config::blank();
    config.set_base("/repo/");

    let anchors = BTreeSet::new();
    let pages = [Rendered {
        source: Path::new("content/index.md"),
        url: "/repo/",
        anchors: &anchors,
        error_page: false,
        html,
    }];

    let _ = links::absolutize(html, &config);
    let _ = links::check(&config, &pages, &BTreeMap::new(), &BTreeSet::new());
});
