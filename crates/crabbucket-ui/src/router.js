// crabbucket client router. GPL-3.0-or-later.
//
// Swaps <main> instead of reloading. A client-side navigation must be at
// least as good as the full page load it replaced, so it also moves focus,
// announces the new page, honours fragments, and restores scroll on Back.
(() => {
  const main = () => document.querySelector('main');
  const NAV = '@NAV@ a';
  const TAB = 'cb-tab:';

  // sessionStorage throws outright in some privacy modes.
  const store = {
    get: (k) => { try { return sessionStorage.getItem(k); } catch { return null; } },
    set: (k, v) => { try { sessionStorage.setItem(k, v); } catch { /* fine */ } },
  };

  const crier = document.createElement('div');
  crier.setAttribute('aria-live', 'polite');
  crier.setAttribute('aria-atomic', 'true');
  crier.style.cssText =
    'position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap';

  // Tabs are CSS-only, so this only remembers a choice; it never enables one.
  const tabs = () => {
    document.querySelectorAll('[data-tab-group]').forEach((group) => {
      const want = store.get(TAB + group.dataset.tabGroup);
      const input = want && [...group.querySelectorAll('input[data-tab-label]')]
        .find((i) => i.dataset.tabLabel === want);
      if (input) input.checked = true;
    });
  };

  const target = (hash) => (hash ? document.getElementById(decodeURIComponent(hash.slice(1))) : null);

  const land = (hash) => {
    const at = target(hash) || main();
    if (!at) return;
    if (!at.hasAttribute('tabindex')) at.setAttribute('tabindex', '-1');
    at.focus({ preventScroll: true });
  };

  const swap = (doc, hash) => {
    main().replaceWith(doc.querySelector('main'));
    document.title = doc.title;
    document.querySelectorAll(NAV).forEach((a) => {
      if (a.pathname === location.pathname) a.setAttribute('aria-current', 'page');
      else a.removeAttribute('aria-current');
    });
    tabs();
    crier.textContent = doc.title;
    land(hash);
  };

  const go = async (url, push, y) => {
    const to = new URL(url, location.href);
    let doc;

    try {
      const res = await fetch(to, { headers: { 'x-crabbucket': '1' } });
      if (!res.ok) throw new Error(res.status);
      doc = new DOMParser().parseFromString(await res.text(), 'text/html');
    } catch {
      location.assign(to);
      return;
    }

    if (push) {
      history.replaceState({ y: scrollY }, '');
      history.pushState({ y: 0 }, '', to);
    }

    const run = () => {
      swap(doc, to.hash);
      const at = target(to.hash);
      if (at) at.scrollIntoView();
      else scrollTo(0, y || 0);
    };

    const animate =
      document.startViewTransition && !matchMedia('(prefers-reduced-motion: reduce)').matches;

    if (animate) document.startViewTransition(run);
    else run();
  };

  addEventListener('DOMContentLoaded', () => { document.body.append(crier); tabs(); });

  addEventListener('change', (e) => {
    const input = e.target.matches?.('input[data-tab-label]') ? e.target : null;
    const group = input?.closest('[data-tab-group]');
    if (group) store.set(TAB + group.dataset.tabGroup, input.dataset.tabLabel);
  });

  addEventListener('click', (e) => {
    if (e.defaultPrevented || e.button || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const a = e.target.closest('a');
    if (!a || a.target || a.hasAttribute('download') || a.origin !== location.origin) return;
    if (a.pathname === location.pathname && !a.hash) return;
    e.preventDefault();
    go(a.href, true);
  });

  addEventListener('popstate', (e) => go(location.href, false, e.state?.y));

  if ('scrollRestoration' in history) history.scrollRestoration = 'manual';
})();
