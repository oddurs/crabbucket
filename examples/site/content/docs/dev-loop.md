+++
title = "The dev loop"
layout = "docs"
order = 7
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

Everything below describes turborust as of **2026-09-08**. It is a separate
project under active development, and it is not a dependency of crabbucket —
so the scaffolded config is checked two ways rather than assumed. A test
asserts its structure always, and runs `turborust plan` against it wherever
turborust is installed.


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

Note what is not there: watch globs. `cargo = "crabbucket-cli"` derives them
from the crate's path-dependency closure.

That derivation only happens when `inputs` is empty, which matters for a
[site crate](../design-systems/): naming `cargo` there would derive the Rust
inputs and then *not* watch the content. `crab new --theme` therefore writes
both lists explicitly, and says why in a comment.

`crab dev` and `crab serve` both exist as commands, and both do the same
thing — print a one-line diagnostic pointing at `turborust up` and exit 2.
A missing command that says where the command went is better than a missing
command.

## The commands

As of 2026-09-08:

```
turborust up [targets…]     start services and keep them running (TUI)
turborust run <task>…       run tasks once, in order, honouring the cache
turborust why <task>        why would this run — or not — right now
turborust doctor            what is costing you seconds on every rebuild
turborust plan              the resolved graph, with derived globs
turborust connect <node>    attach this terminal to a running process
turborust init              scaffold a config from your Cargo workspace
turborust schema            JSON Schema for turborust.toml
turborust completions <sh>  shell completion script
turborust man               the man page
turborust clean             drop cached task results
```

`turborust schema` is the one worth knowing about from here: it emits a JSON
Schema for `turborust.toml`, which is a better long-term answer than the
structural test crabbucket currently keeps.

## A note on licences

turborust is MIT OR Apache-2.0 and crabbucket is GPL-3.0-or-later, so code
may move from turborust into crabbucket but not the other way. Worth knowing
before anything useful gets written in the wrong repository.

## Dogfooding, structurally

crabbucket's own site is built by crabbucket and developed under turborust,
and turborust's example stack is served the same way. Neither project has a
demo that exists only to be demoed.
