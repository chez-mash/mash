// Right-click menus. A page names the lines of a menu and what each does;
// in the desktop app the app shows them as the system's own menu at the
// pointer (desktop/src-tauri/src/context.rs) and says which was chosen by its
// place in the list, and in a browser the dashboard draws the same lines
// itself (components/ContextMenu.svelte). Either way the page keeps what a
// line does, so one list serves both, as VS Code's menus do.

import { inDesktopApp } from "./shareLink";

export interface MenuLine {
  readonly title: string;
  readonly run: () => void;
  readonly enabled?: boolean;
}
/** A line, or a rule between groups of lines. */
export type MenuEntry = MenuLine | "separator";

/** The event the app sends with the chosen line's place (context.rs). */
const CHOSEN_EVENT = "mash:context";

type Invoke = (command: string, args: object) => Promise<unknown>;

/** The app's commands, inside the desktop app and nowhere else. */
function appCommands(scope: object): Invoke | undefined {
  if (!inDesktopApp(scope)) return undefined;
  const invoke = (scope as { __TAURI_INTERNALS__?: { invoke?: Invoke } }).__TAURI_INTERNALS__
    ?.invoke;
  return typeof invoke === "function" ? invoke : undefined;
}

/** What the app is sent for each entry: a title, or nothing for a rule. */
export function linesFor(entries: readonly MenuEntry[]): object[] {
  return entries.map((entry) =>
    entry === "separator" ? {} : { title: entry.title, enabled: entry.enabled ?? true },
  );
}

class ContextMenu {
  /** The menu the dashboard draws in a browser, while it is open. */
  open = $state<{ x: number; y: number; entries: readonly MenuEntry[] } | null>(null);
  #waiting: readonly MenuEntry[] = [];

  /** Opens `entries` at the pointer of a right-click. */
  show(event: MouseEvent, entries: readonly MenuEntry[], scope: object = globalThis): void {
    if (!entries.some((entry) => entry !== "separator")) return;
    event.preventDefault();
    const at = { x: event.clientX, y: event.clientY };
    const invoke = appCommands(scope);
    if (!invoke) {
      this.open = { ...at, entries };
      return;
    }
    this.#waiting = entries;
    // An app too old for the command still gets a menu, drawn by the page.
    invoke("context_menu", { lines: linesFor(entries) }).catch(() => {
      this.open = { ...at, entries };
    });
  }

  /** The app's answer: the place of the line chosen from its menu. */
  chosen(place: number): void {
    const entry = this.#waiting[place];
    this.#waiting = [];
    if (entry && entry !== "separator" && entry.enabled !== false) entry.run();
  }

  close(): void {
    this.open = null;
  }
}

export const contextMenu = new ContextMenu();

/** Listens for the app's answers; returns what stops listening. */
export function installContextMenu(scope: Window = window): () => void {
  const listener = (event: Event) => {
    const place = Number((event as CustomEvent).detail);
    if (Number.isInteger(place)) contextMenu.chosen(place);
  };
  scope.addEventListener(CHOSEN_EVENT, listener);
  return () => scope.removeEventListener(CHOSEN_EVENT, listener);
}
