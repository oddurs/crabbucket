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
//! What `crab new` writes has to build, and the config it writes for another
//! project has to be one that project accepts.
//!
//! The second half is the awkward one.  turborust is not a dependency and is
//! under active development, so a schema change there would otherwise reach
//! crabbucket's users as a broken scaffold rather than as a failing build.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scaffolds a site into a directory of its own.
fn scaffold(args: &[&str]) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let dir = std::env::temp_dir().join(format!(
        "crabbucket-scaffold-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let _ = fs::remove_dir_all(&dir);

    let out = Command::new(env!("CARGO_BIN_EXE_crab"))
        .arg("new")
        .arg(&dir)
        .args(args)
        .output()
        .expect("could not run crab");

    assert!(
        out.status.success(),
        "crab new failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    dir
}

fn build(dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crab"))
        .arg("build")
        .arg(dir)
        .output()
        .expect("could not run crab")
}

#[test]
fn a_scaffolded_site_builds_with_no_edits() {
    let dir = scaffold(&[]);
    let out = build(&dir);

    assert!(
        out.status.success(),
        "the scaffold does not build: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    for path in ["index.html", "docs/index.html", "404.html", "site.css"] {
        assert!(
            dir.join("dist").join(path).is_file(),
            "{path} was not written"
        );
    }

    // The scaffold is silent about search and the router, so neither ships.
    assert!(!dir.join("dist/router.js").exists());
    assert!(!dir.join("dist/search.json").exists());
}

#[test]
fn the_scaffolded_site_passes_its_own_link_checking() {
    // `crab build` fails on a dead link, so a successful build is the
    // assertion.  This exists to say so out loud: a scaffold that ships a
    // dead link teaches its lesson on somebody's first minute.
    let dir = scaffold(&[]);
    assert!(build(&dir).status.success());
}

#[test]
fn asking_for_the_router_gets_the_router() {
    let dir = scaffold(&["--router"]);
    assert!(build(&dir).status.success());
    assert!(dir.join("dist/router.js").is_file());
}

#[test]
fn a_base_path_reaches_every_generated_link() {
    let dir = scaffold(&["--base", "/somewhere/"]);
    assert!(build(&dir).status.success());

    let html = fs::read_to_string(dir.join("dist/index.html")).expect("no index");
    assert!(html.contains("\"/somewhere/site.css\""), "got {html}");
    assert!(html.contains("\"/somewhere/docs/\""), "got {html}");
}

#[test]
fn scaffolding_over_an_existing_directory_is_refused() {
    let dir = scaffold(&[]);

    let out = Command::new(env!("CARGO_BIN_EXE_crab"))
        .arg("new")
        .arg(&dir)
        .output()
        .expect("could not run crab");

    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--force"));

    let forced = Command::new(env!("CARGO_BIN_EXE_crab"))
        .arg("new")
        .arg(&dir)
        .arg("--force")
        .output()
        .expect("could not run crab");

    assert!(forced.status.success(), "--force should allow it");
}

// ------------------------------------------------------------- turborust

/// The scaffolded turborust config, as TOML.
fn turborust_config() -> toml::Table {
    let dir = scaffold(&[]);
    let text = fs::read_to_string(dir.join("turborust.toml")).expect("no turborust.toml");
    text.parse()
        .expect("the scaffolded turborust.toml is not valid TOML")
}

#[test]
fn the_scaffolded_turborust_config_has_the_shape_turborust_documents() {
    let config = turborust_config();

    let task = config["tasks"]["site"]
        .as_table()
        .expect("[tasks.site] is missing");
    assert!(task["cmd"].as_str().is_some(), "a task needs a cmd");
    assert!(
        task["inputs"].as_array().is_some(),
        "a task without inputs is never cached"
    );
    assert!(task["outputs"].as_array().is_some());

    let service = config["services"]["web"]
        .as_table()
        .expect("[services.web] is missing");
    assert_eq!(
        service["depends_on"].as_array().map(Vec::len),
        Some(1),
        "the web service must wait for the build"
    );

    let serve = service["serve"]
        .as_table()
        .expect("the web service must serve something");
    assert_eq!(serve["dir"].as_str(), Some("dist"));
    assert!(serve["port"].as_integer().is_some());

    let overlay = config["overlay"].as_table().expect("[overlay] is missing");
    for key in ["position", "emoji", "theme", "errors"] {
        assert!(overlay.contains_key(key), "the overlay has no {key}");
    }
}

#[test]
fn turborust_itself_accepts_the_config_we_ship_it() {
    // The structural test above catches our own typos.  Only turborust can
    // catch turborust changing, so run it where it exists.
    let Ok(probe) = Command::new("turborust").arg("--version").output() else {
        eprintln!("skipping: turborust is not on PATH");
        return;
    };

    if !probe.status.success() {
        eprintln!("skipping: turborust is not usable");
        return;
    }

    let dir = scaffold(&[]);
    let planned = Command::new("turborust")
        .arg("-C")
        .arg(&dir)
        .arg("plan")
        .output()
        .expect("turborust failed to run");

    assert!(
        planned.status.success(),
        "turborust rejects the config crab new writes:\n{}",
        String::from_utf8_lossy(&planned.stderr)
    );
}
