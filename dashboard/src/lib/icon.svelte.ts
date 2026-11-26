// How heavy an icon is drawn. Phosphor draws every glyph in several weights,
// and the weight is chosen here and nowhere else, from the icon's size and
// what it is saying: a small icon is bold so it holds up beside 12px text, a
// large decorative one is light, the current item is filled, and a warning,
// a failure or the icon on a primary button is bold. The weights themselves
// are the --icon-weight-* roles in theme/tokens.css, so a theme can draw its
// icons heavier without touching a component.

/** The weights the dashboard draws. Phosphor's thin and duotone are cut from the build (vite.config.ts). */
export type IconWeight = "light" | "regular" | "bold" | "fill";

/**
 * rest: an ordinary icon, weighted by its size. on: the current navigation
 * item, the selected tab or segment, a switch that is on. strong: a warning or
 * failure, or the icon on a primary button.
 */
export type IconState = "rest" | "on" | "strong";

export interface IconWeights {
  /** Icons up to SMALL_MAX px. */
  readonly small: IconWeight;
  /** Icons between the two limits. */
  readonly base: IconWeight;
  /** Decorative icons from LARGE_MIN px. */
  readonly large: IconWeight;
  readonly on: IconWeight;
  readonly strong: IconWeight;
}

const SMALL_MAX = 15;
const LARGE_MIN = 28;

/** The roles in theme/tokens.css, by the field each one fills. */
export const weightTokens = {
  small: "--icon-weight-small",
  base: "--icon-weight",
  large: "--icon-weight-large",
  on: "--icon-weight-on",
  strong: "--icon-weight-strong",
} as const satisfies Record<keyof IconWeights, string>;

/** The light theme's weights, used until the theme has been read (theme.test.ts keeps them equal). */
export const defaultIconWeights: IconWeights = {
  small: "bold",
  base: "regular",
  large: "light",
  on: "fill",
  strong: "bold",
};

const weights: readonly string[] = ["light", "regular", "bold", "fill"];

function isWeight(value: string): value is IconWeight {
  return weights.includes(value);
}

/** The weight for an icon of `size` px in `state`. */
export function iconWeight(
  size: number,
  state: IconState = "rest",
  from: IconWeights = defaultIconWeights,
): IconWeight {
  if (state === "on") return from.on;
  if (state === "strong") return from.strong;
  if (size <= SMALL_MAX) return from.small;
  return size >= LARGE_MIN ? from.large : from.base;
}

/** The theme's weights from its computed style; a role that is missing keeps its default. */
export function readIconWeights(style: Pick<CSSStyleDeclaration, "getPropertyValue">): IconWeights {
  const read = (field: keyof IconWeights): IconWeight => {
    const value = style.getPropertyValue(weightTokens[field]).trim();
    return isWeight(value) ? value : defaultIconWeights[field];
  };
  return {
    small: read("small"),
    base: read("base"),
    large: read("large"),
    on: read("on"),
    strong: read("strong"),
  };
}

class IconTheme {
  weights = $state<IconWeights>(defaultIconWeights);

  refresh(root: Element | undefined = globalThis.document?.documentElement): void {
    if (root) this.weights = readIconWeights(getComputedStyle(root));
  }
}

/** The weights of the theme now showing. */
export const iconTheme = new IconTheme();

/** Reads the theme's icon weights, and again whenever the theme changes. */
export function installIconTheme(): void {
  iconTheme.refresh();
  const reread = () => iconTheme.refresh();
  globalThis.matchMedia?.("(prefers-color-scheme: dark)").addEventListener("change", reread);
  new MutationObserver(reread).observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
}
