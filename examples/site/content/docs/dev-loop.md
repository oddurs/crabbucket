+++
title = "The dev loop"
layout = "docs"
+++

# The dev loop

There is no `crab dev`, and there will not be one.

```sh
turborust up
```

## Why not

[turborust](https://github.com/oddurs/turborust) is a dev orchestrator for
full-Rust stacks. It already does every part of the job:

- **Watching.** It asks `cargo metadata` for a crate's path-dependency
  closure and derives the watch globs from it. Edit `crates/crabbucket` or
  `design/tokens.toml` and the site rebuilds, without anyone having to
  remember to list those paths — which is how the shared-crate problem
  happens everywhere else.
- **Caching.** Task keys are blake3 hashes over inputs, command, declared env
  and upstream keys. Content hashes rather than mtimes, so `git checkout` does
  not invalidate the world.
- **Serving.** `serve = { dir = "dist", port = 8790 }` hosts the output
  in-process with live reload.
- **Errors where you are looking.** rustc diagnostics are parsed out of
  cargo's output and shown in a browser overlay, with the code frame and a
  `vscode://` link, over a blur of the page you were reading.

Writing a second, worse version of that inside crabbucket would be the least
defensible thing in the tree.

## The config

`turborust.toml`, at the repository root:

```toml
[overlay]
position = "bottom-right"
emoji = "🦀"
theme = "auto"
errors = "overlay"

[tasks.site]
cargo = "crabbucket-cli"
cmd = "cargo run -q -p crabbucket-cli -- build examples/site"

[services.web]
depends_on = ["site"]
serve = { dir = "examples/site/dist", port = 8790 }
```

Note what is not there: watch globs. `cargo = "crabbucket-cli"` derives them.

`crab dev` and `crab serve` both exist as commands, and both do the same
thing — print a one-line diagnostic pointing at `turborust up` and exit 2.
A missing command that says where the command went is better than a missing
command.

## Dogfooding, structurally

crabbucket's own site is built by crabbucket and developed under turborust,
and turborust's example stack is served the same way. Neither project has a
demo that exists only to be demoed.
