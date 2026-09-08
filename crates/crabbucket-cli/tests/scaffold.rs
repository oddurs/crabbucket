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

// ------------------------------------------------------------ with a theme

#[test]
fn a_theme_makes_the_site_a_crate_instead_of_a_content_directory() {
    // A design system is a crate you depend on, so a site that depends on one
    // is a program that calls the build.  The two shapes need different build
    // commands, and shipping the content-shaped config into a crate-shaped
    // site rebuilds nothing when the Rust changes.
    let dir = scaffold(&["--theme", "my-house-style"]);

    assert!(dir.join("Cargo.toml").is_file(), "no manifest");
    assert!(dir.join("src/main.rs").is_file(), "nothing to run");

    let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("no manifest");
    assert!(
        manifest.contains("my-house-style = \"*\""),
        "got {manifest}"
    );
    assert!(
        manifest.contains("crabbucket ="),
        "the framework is not a dependency"
    );

    let main = fs::read_to_string(dir.join("src/main.rs")).expect("no main");
    assert!(
        main.contains("use my_house_style::Standard as Design"),
        "got {main}"
    );
    assert!(
        main.contains("crabbucket::build"),
        "the site does not build itself: {main}"
    );

    let ignored = fs::read_to_string(dir.join(".gitignore")).expect("no gitignore");
    assert!(
        ignored.contains("/target"),
        "a crate ignores its target directory"
    );
}

#[test]
fn a_git_url_becomes_a_git_dependency() {
    let dir = scaffold(&["--theme", "https://github.com/me/house-style.git"]);
    let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("no manifest");

    assert!(
        manifest.contains("git = \"https://github.com/me/house-style.git\""),
        "{manifest}"
    );
    assert!(
        !manifest.contains("\"*\""),
        "a git dependency needs no version: {manifest}"
    );
}

#[test]
fn a_crate_shaped_site_watches_its_rust_and_runs_itself() {
    // This is the finding that prompted the option: a content-shaped config
    // in a crate-shaped site rebuilt nothing when the stylesheet changed.
    let dir = scaffold(&["--theme", "my-house-style"]);
    let text = fs::read_to_string(dir.join("turborust.toml")).expect("no turborust.toml");
    let config: toml::Table = text.parse().expect("not valid TOML");

    let task = config["tasks"]["site"]
        .as_table()
        .expect("[tasks.site] is missing");
    assert_eq!(
        task["cmd"].as_str(),
        Some("cargo run -q"),
        "a crate does not run `crab build`"
    );

    let inputs: Vec<&str> = task["inputs"]
        .as_array()
        .expect("no inputs")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    for glob in ["src/**", "Cargo.toml", "content/**"] {
        assert!(inputs.contains(&glob), "{glob} is not an input: {inputs:?}");
    }

    let watch: Vec<&str> = config["services"]["web"]["watch"]
        .as_array()
        .expect("no watch")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        watch.contains(&"src/**"),
        "the Rust is not watched: {watch:?}"
    );
}

#[test]
fn a_crate_shaped_site_deploys_by_running_itself() {
    let dir = scaffold(&["--theme", "my-house-style"]);
    let workflow =
        fs::read_to_string(dir.join(".github/workflows/pages.yml")).expect("no workflow");

    assert!(workflow.contains("cargo run --release"), "got {workflow}");
    assert!(
        !workflow.contains("cargo install"),
        "a crate does not install crab: {workflow}"
    );
}

#[test]
fn the_generated_crate_compiles_and_builds_its_site() {
    // Slow: it compiles crabbucket and its dependencies into a scratch
    // directory.  Off by default; CI can afford the time.
    if std::env::var_os("CRABBUCKET_SLOW_TESTS").is_none() {
        eprintln!("skipping: set CRABBUCKET_SLOW_TESTS=1 to compile the scaffold");
        return;
    }

    let dir = scaffold(&["--theme", "crabbucket-ui"]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // A real site would depend on published crates.  This one points at the
    // working copy, so the test checks today's API rather than a release's.
    let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("no manifest");
    let manifest = manifest
        .replace(
            "crabbucket = \"0.1\"",
            &format!(
                "crabbucket = {{ path = {:?} }}",
                root.join("crates/crabbucket")
            ),
        )
        .replace(
            "crabbucket-ui = \"*\"",
            &format!(
                "crabbucket-ui = {{ path = {:?} }}",
                root.join("crates/crabbucket-ui")
            ),
        );

    fs::write(dir.join("Cargo.toml"), format!("{manifest}\n[workspace]\n")).expect("write");

    let out = Command::new(env!("CARGO"))
        .arg("run")
        .arg("-q")
        .current_dir(&dir)
        .env("CARGO_TARGET_DIR", root.join("target/scaffold"))
        .output()
        .expect("cargo failed to run");

    assert!(
        out.status.success(),
        "the scaffolded crate does not build:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(dir.join("dist/index.html").is_file(), "it built nothing");
}

// ------------------------------------------------------------- turborust

/// The scaffolded turborust config, as TOML.
fn turborust_config(args: &[&str]) -> toml::Table {
    let dir = scaffold(args);
    let text = fs::read_to_string(dir.join("turborust.toml")).expect("no turborust.toml");
    text.parse()
        .expect("the scaffolded turborust.toml is not valid TOML")
}

#[test]
fn the_scaffolded_turborust_config_has_the_shape_turborust_documents() {
    let config = turborust_config(&[]);

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

    // Both shapes, because they are two different configs.
    for args in [&[][..], &["--theme", "my-house-style"][..]] {
        let dir = scaffold(args);
        let planned = Command::new("turborust")
            .arg("-C")
            .arg(&dir)
            .arg("plan")
            .output()
            .expect("turborust failed to run");

        assert!(
            planned.status.success(),
            "turborust rejects the config `crab new {args:?}` writes:\n{}",
            String::from_utf8_lossy(&planned.stderr)
        );
    }
}
