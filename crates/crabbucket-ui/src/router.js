// crabbucket client router. GPL-3.0-or-later.
(() => {
  const swap = (doc) => {
    document.querySelector('main').replaceWith(doc.querySelector('main'));
    document.title = doc.title;
    document.querySelectorAll('@NAV@ a').forEach((a) => {
      a.toggleAttribute('aria-current', a.pathname === location.pathname);
    });
  };
  const go = async (url, push) => {
    const res = await fetch(url, { headers: { 'x-crabbucket': '1' } });
    if (!res.ok) { location.assign(url); return; }
    const doc = new DOMParser().parseFromString(await res.text(), 'text/html');
    if (push) history.pushState(null, '', url);
    document.startViewTransition ? document.startViewTransition(() => swap(doc)) : swap(doc);
    scrollTo(0, 0);
  };
  addEventListener('click', (e) => {
    if (e.defaultPrevented || e.button || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const a = e.target.closest('a');
    if (!a || a.target || a.hasAttribute('download') || a.origin !== location.origin) return;
    if (a.pathname === location.pathname) return;
    e.preventDefault();
    go(a.href, true).catch(() => location.assign(a.href));
  });
  addEventListener('popstate', () => go(location.href, false).catch(() => location.reload()));
})();
