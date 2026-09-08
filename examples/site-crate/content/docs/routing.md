+++
title = "The route table"
+++

# The route table

Every route in `content/` becomes a variant. `content/docs/routing.md` is
`Route::DocsRouting`; `content/index.md` is `Route::Index`.

Two routes that would share a variant name fail the build naming both, because
picking one silently would mean a link that compiles and goes to the wrong
page.
