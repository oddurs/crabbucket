// crabbucket search client. GPL-3.0-or-later.
//
// The index is every page as JSON, scanned linearly. For tens of pages that is
// faster than parsing an inverted index would be, and a tenth of the code.
// It is fetched on the first interaction, not on page load, so a reader who
// never searches never pays for it.
(() => {
  const form = document.querySelector('@FORM@');
  if (!form) return;

  const input = form.querySelector('input[type="search"]');
  const list = form.querySelector('@RESULTS@');
  if (!input || !list) return;

  // The markup ships hidden so that a reader without script sees no search
  // box rather than one that does nothing.
  form.hidden = false;

  const LIMIT = 8;
  let index;
  let at = -1;

  // The index URL comes from the form rather than from this file, because it
  // carries the site's base path and this file does not know there is one.
  const load = () => (index ||= fetch(form.dataset.index)
    .then((res) => res.json())
    .catch(() => (index = [])));

  const score = (page, needle) => {
    let found = 0;
    if (page.t.toLowerCase().includes(needle)) found += 8;
    for (const [text] of page.h) if (text.toLowerCase().includes(needle)) found += 4;
    if (page.b.toLowerCase().includes(needle)) found += 1;
    return found;
  };

  // The heading nearest the match, so a result lands where the words are.
  const anchor = (page, needle) => {
    const hit = page.h.find(([text]) => text.toLowerCase().includes(needle));
    return hit ? `${page.u}#${hit[1]}` : page.u;
  };

  const render = (results, needle) => {
    at = -1;
    list.replaceChildren();
    list.hidden = !results.length;

    results.forEach((page) => {
      const a = document.createElement('a');
      a.href = anchor(page, needle);
      a.className = '@RESULT@';
      a.textContent = page.t;
      list.append(a);
    });
  };

  const search = async () => {
    const needle = input.value.trim().toLowerCase();
    if (needle.length < 2) return render([], needle);

    const pages = await load();
    const found = pages
      .map((page) => [score(page, needle), page])
      .filter(([found]) => found > 0)
      .sort((a, b) => b[0] - a[0])
      .slice(0, LIMIT)
      .map(([, page]) => page);

    render(found, needle);
  };

  const move = (by) => {
    const links = [...list.children];
    if (!links.length) return;
    at = (at + by + links.length + 1) % (links.length + 1);
    links.forEach((a, i) => a.setAttribute('aria-selected', i === at));
    if (at >= 0) links[at].focus();
    else input.focus();
  };

  input.addEventListener('input', search);
  input.addEventListener('focus', load, { once: true });
  form.addEventListener('submit', (e) => {
    e.preventDefault();
    if (list.firstChild) location.assign(list.firstChild.href);
  });

  form.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); move(1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); move(-1); }
    else if (e.key === 'Escape') { input.value = ''; render([], ''); input.blur(); }
  });

  addEventListener('keydown', (e) => {
    if (e.key !== '/' || e.metaKey || e.ctrlKey) return;
    if (/^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName)) return;
    e.preventDefault();
    input.focus();
  });

  addEventListener('click', (e) => { if (!form.contains(e.target)) list.hidden = true; });
})();
