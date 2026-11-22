// Runs in the window before each page loads, the dashboard included.

// The dashboard shares mash:// links instead of its private address while
// this is set (dashboard/src/lib/shareLink.ts).
window.__MASH_DESKTOP__ = true;

// The title bar follows the dashboard's light or dark theme, which the
// dashboard marks on the root element. Setting the window's theme is the one
// app command the dashboard may call (capabilities/dashboard.json).
(() => {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  // The search panel follows the system's light or dark mode, not the
  // dashboard's (dashboard/src/components/QuickSearch.svelte).
  if (!invoke || location.hash.startsWith("#/quick")) return;
  let shown = null;
  const follow = () => {
    const theme = document.documentElement.dataset.theme === "light" ? "light" : "dark";
    if (theme === shown) return;
    shown = theme;
    invoke("plugin:window|set_theme", { value: theme }).catch(() => {});
  };
  const watch = () => {
    follow();
    new MutationObserver(follow).observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", watch);
  else watch();
})();
