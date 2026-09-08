// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The `+++' split and the TOML behind it, on arbitrary bytes.  This is the
// one with a bug in its history: the closing fence is found by its newline, so
// a file with CRLF line endings kept a carriage return that TOML rejected --
// which passed review on three platforms and was found by running on a fourth.

#![no_main]

use std::fs;
use std::path::PathBuf;

use crabbucket::Config;
use crabbucket::content::Collection;
use crabbucket::directive::{Context, Data, Directives};
use crabbucket::theme::PageMeta;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|source: &str| {
    // `Collection::load' reads a directory, so the input has to be a file.
    // One scratch directory per process, reused, rather than one per case.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fuzz-scratch")
        .join(std::process::id().to_string());

    let content = dir.join("content");
    if fs::create_dir_all(&content).is_err() {
        return;
    }

    if fs::write(content.join("page.md"), source).is_err() {
        return;
    }

    let config = Config::blank();
    let data = Data::default();
    let context = Context::new(&config, &data);

    let _ = Collection::<PageMeta<(), ()>>::load(&content, &Directives::new(), &context);
});
