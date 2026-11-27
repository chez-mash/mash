// Which window the dashboard is drawn in. The desktop app says which system
// it runs on before the page loads (desktop/src-tauri/src/frame.rs); in a
// browser there is no such word and the page is a page. The shell is set on
// the document as data-shell, so the stylesheet can fit that system's frame:
// on macOS the top bar is the window's title bar, with the traffic lights
// over the sidebar's colour at its left.

const shells = ["macos", "windows", "linux", "browser"] as const;
export type Shell = (typeof shells)[number];

/** The shell `scope` says it is in; anything unknown is a browser. */
export function shellOf(scope: unknown = globalThis): Shell {
  const said = (scope as { __MASH_SHELL__?: unknown }).__MASH_SHELL__;
  return shells.includes(said as Shell) ? (said as Shell) : "browser";
}

/** The shell this page is in. */
export const shell: Shell = shellOf();

/** Marks the document with its shell before the dashboard first draws. */
export function installShell(root: HTMLElement = document.documentElement): void {
  root.dataset.shell = shell;
}
