// The reader's choice of light or dark. "system" follows the operating
// system through the theme's prefers-color-scheme rule; "light" and "dark"
// pin one theme by setting data-theme on the document, which tokens.css reads.
// The choice is a per-browser convenience kept in localStorage: when storage
// is unavailable the dashboard simply follows the system.

export const appearances = ["system", "light", "dark"] as const;
export type Appearance = (typeof appearances)[number];

const KEY = "mash.appearance";

export function parseAppearance(value: unknown): Appearance {
  return appearances.includes(value as Appearance) ? (value as Appearance) : "system";
}

function stored(): Appearance {
  try {
    return parseAppearance(globalThis.localStorage?.getItem(KEY));
  } catch {
    return "system";
  }
}

/** Sets or clears data-theme on the document for one choice. */
export function applyAppearance(
  choice: Appearance,
  root: HTMLElement = document.documentElement,
): void {
  if (choice === "system") delete root.dataset.theme;
  else root.dataset.theme = choice;
}

class AppearanceState {
  current = $state<Appearance>(stored());

  set(choice: Appearance): void {
    this.current = choice;
    applyAppearance(choice);
    try {
      if (choice === "system") globalThis.localStorage?.removeItem(KEY);
      else globalThis.localStorage?.setItem(KEY, choice);
    } catch {
      // Storage is unavailable; the choice lasts until the page reloads.
    }
  }
}

export const appearance = new AppearanceState();

/** Applies the saved choice before the dashboard first draws. */
export function installAppearance(): void {
  applyAppearance(appearance.current);
}
