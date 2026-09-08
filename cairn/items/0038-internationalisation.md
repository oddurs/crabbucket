---
id: 38
title: Internationalisation
type: feature
status: backlog
labels:
- out-of-scope
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
