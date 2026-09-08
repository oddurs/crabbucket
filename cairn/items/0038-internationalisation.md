---
id: 38
title: Internationalisation
type: feature
status: backlog
created: 2026-09-08
updated: 2026-09-08
priority: p3
effort: xl
area: content
---

## Problem

Same as wasm islands: excluded in `doc/DESIGN` section 11, filed so the
exclusion is visible rather than implicit.

## Proposal

Not now, and the reason is worth stating precisely, because "not needed yet"
is not the whole of it.

Done properly, i18n touches everything: routes gain a locale segment, the
route type gains a dimension, collections are per-locale, navigation needs a
language switcher, feeds and sitemaps multiply, and every URL in the system
needs to know which locale it belongs to. It is not a feature that can be
added at the edge — it changes the core types.

Which means the honest sequencing is: do it properly, or do not do it. A
half-measure — a `lang` field in frontmatter and hope — would make the
typed-route work harder later and would not actually serve a translated site.

Reopen when there is a real site to translate.

## Acceptance criteria

- [ ] Left closed until a real translated site exists
- [ ] If reopened, done as a core change, not an edge one

## 2026-09-08

Not scheduled, but the cost estimate has changed and the reason is worth recording.

Two findings from the survey. Zola's taxonomies carry a lang field and generate a route per term per language, so once there is a typed generated-routes seam (0061) the per-language route generation is the same machinery rather than new machinery. And Pagefind partitions its whole search index by language -- every chunk hash is prefixed with a language_ref -- which is the piece that would otherwise be hardest, and 0060 is adopting that index shape anyway.

So the honest position: i18n is cheaper after 0060 and 0061 than it looks now, and it is still XL because it touches Url, the nav, feeds, the sitemap, and every design system's layout. It stays unscheduled because the sites this framework exists for are in one language, and a feature nobody here needs is a feature nobody here will keep correct. Reopen when a real site needs a second language.
