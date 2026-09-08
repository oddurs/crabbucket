// crabbucket client router. GPL-3.0-or-later.
//
// Swaps <main> instead of reloading. A client-side navigation must be at
// least as good as the full page load it replaced, so it also moves focus,
// announces the new page, honours fragments, and restores scroll on Back.
(() => {
  const main = () => document.querySelector('main');
  const NAV = '@NAV@ a';
  const TOC = '@TOC@';
  const COPY = '@COPY@';
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
    spy();
    copy();
    crier.textContent = doc.title;
    land(hash);
  };

  // A small in-memory cache, so a hover can pay for a click. It is dropped
  // with the page: a cache that outlives the document serves stale content
  // after a deploy, which is a worse bug than a slow navigation.
  const CACHE = 8;
  const pages = new Map();

  const load = (to) => {
    const key = to.href.split('#')[0];
    let pending = pages.get(key);

    if (!pending) {
      pending = fetch(to, { headers: { 'x-crabbucket': '1' } })
        .then((res) => {
          if (!res.ok) throw new Error(res.status);
          return res.text();
        })
        .then((text) => new DOMParser().parseFromString(text, 'text/html'))
        .catch((err) => { pages.delete(key); throw err; });

      pages.set(key, pending);
      if (pages.size > CACHE) pages.delete(pages.keys().next().value);
    }

    return pending;
  };

  const go = async (url, push, y) => {
    const to = new URL(url, location.href);
    let doc;

    try {
      doc = await load(to);
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

  // Marks the contents entry for the section in view. The contents list is
  // plain anchors and works without this; it only ever adds emphasis.
  let watching;
  const spy = () => {
    watching?.disconnect();
    const links = [...document.querySelectorAll(TOC)];
    if (!links.length) return;

    const seen = new Set();
    watching = new IntersectionObserver((entries) => {
      entries.forEach((e) => (e.isIntersecting ? seen.add(e.target.id) : seen.delete(e.target.id)));
      links.forEach((a) => {
        if (seen.has(decodeURIComponent(a.hash.slice(1)))) a.setAttribute('aria-current', 'true');
        else a.removeAttribute('aria-current');
      });
    }, { rootMargin: '0px 0px -70% 0px' });

    links.forEach((a) => {
      const at = document.getElementById(decodeURIComponent(a.hash.slice(1)));
      if (at) watching.observe(at);
    });
  };

  // Copy buttons are added here rather than emitted by the build, so a reader
  // with scripting off gets no button rather than a button that does nothing.
  const copy = () => {
    document.querySelectorAll('pre:not([data-copy])').forEach((pre) => {
      pre.dataset.copy = '';
      const button = document.createElement('button');
      button.type = 'button';
      button.className = COPY;
      button.setAttribute('aria-label', 'Copy code');
      button.textContent = 'Copy';

      button.addEventListener('click', async () => {
        // Copying the shell prompt is the small annoyance this removes.
        const text = pre.innerText.replace(/^\$ /gm, '');
        try {
          await navigator.clipboard.writeText(text);
          button.textContent = 'Copied';
          crier.textContent = 'Copied to clipboard';
        } catch {
          button.textContent = 'Press Ctrl-C';
        }
        setTimeout(() => { button.textContent = 'Copy'; }, 2000);
      });

      pre.append(button);
    });
  };

  addEventListener('DOMContentLoaded', () => {
    document.body.append(crier);
    tabs();
    spy();
    copy();
  });

  addEventListener('change', (e) => {
    const input = e.target.matches?.('input[data-tab-label]') ? e.target : null;
    const group = input?.closest('[data-tab-group]');
    if (group) store.set(TAB + group.dataset.tabGroup, input.dataset.tabLabel);
  });

  // Prefetching is worth about 160ms of the 230ms a page fetch costs from a
  // CDN, which is the difference between a navigation that feels instant and
  // one that does not. It is skipped entirely on a connection that says it
  // would rather not.
  const link = (node) => {
    const a = node?.closest?.('a');
    return a && !a.target && !a.hasAttribute('download') && a.origin === location.origin
      && a.pathname !== location.pathname
      ? a
      : null;
  };

  const net = navigator.connection || {};
  const eager = !net.saveData && !/2g/.test(net.effectiveType || '');

  let waiting;
  addEventListener('pointerover', (e) => {
    const a = link(e.target);
    if (!a || !eager) return;
    clearTimeout(waiting);
    waiting = setTimeout(() => load(new URL(a.href)).catch(() => {}), 65);
  });

  addEventListener('pointerout', () => clearTimeout(waiting));

  if (eager && /4g/.test(net.effectiveType || '4g')) {
    const ahead = new IntersectionObserver((entries) => {
      entries.forEach((e) => {
        if (!e.isIntersecting) return;
        ahead.unobserve(e.target);
        load(new URL(e.target.href)).catch(() => {});
      });
    });

    addEventListener('DOMContentLoaded', () => {
      document.querySelectorAll(NAV).forEach((a) => link(a) && ahead.observe(a));
    });
  }

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
