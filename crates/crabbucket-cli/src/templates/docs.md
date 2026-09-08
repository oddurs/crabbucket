+++
title = "Documentation"
layout = "docs"
nav_order = 2
nav_label = "Docs"
+++

# Documentation

Add a page by putting a Markdown file in `content/`. Its path becomes its
route, so `content/docs/install.md` is served at `docs/install/`.

Every page needs a `title`. A page can also set:

| Field | Meaning |
|---|---|
| `layout` | `page`, `docs` or `landing`. Unknown values fail the build. |
| `nav_order` | Put the page in the top navigation, at this position. |
| `nav_label` | A shorter label for the navigation than the title. |
| `description` | Overrides the site description for this page. |
| `draft` | Skip the page entirely. |

Links between pages are checked when the site is built. A link to a page
that does not exist, or to a heading that does not exist, fails the build.
