/**
 * Links a reader copies to come back to a page of the dashboard.
 *
 * In a browser the link is the page's own address. In the Mash desktop app the
 * dashboard is served on a private port that changes on every start, so an
 * address there would stop working; the app instead registers mash:// links,
 * which open the same page in it. The app marks its window by setting
 * `__MASH_DESKTOP__` before the dashboard loads.
 */

/** Whether the dashboard is running inside the Mash desktop app. */
export function inDesktopApp(scope: object = globalThis): boolean {
  return (scope as { __MASH_DESKTOP__?: unknown }).__MASH_DESKTOP__ === true;
}

/**
 * The link to share for a route such as `#/chat/abc?node=laptop`.
 * `base` is the page's address without its hash.
 */
export function shareLink(route: string, base: string, desktop = inDesktopApp()): string {
  if (desktop) {
    // mash://chat/abc?node=laptop: the route without its leading "#/".
    const path = route.replace(/^#?\/?/, "");
    return `mash://${path}`;
  }
  return new URL(route, base).href;
}
