+++
title = "How it works"
nav_order = 2
nav_label = "How"
+++

# How it works

```rust
use crabbucket::Url;

Url::new(config, Route::DocsRouting.path())
```

Rename `content/docs/routing.md` and `Route::DocsRouting` stops existing, so
every reference to it fails to compile — which is a better time to find out
than after the build has finished and the site has shipped.
