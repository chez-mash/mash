/**
 * Mash's tagline: the description of any page that gives none, and the
 * tagline in the landing page's title and link previews. astro.config.mjs
 * repeats it as Starlight's default description.
 */
export const tagline = "Supercharged context for your agents.";

/** The landing page's headline, one line each. */
export const headline = ["Your agents forget.", "Mash doesn’t."] as const;

/**
 * The Mash extension's page in the Raycast Store, or nothing until the Store
 * lists it (raycast/extensions#33948). Set, the landing page offers it to Mac
 * visitors as a button; unset, a quiet link leads to the extension's source.
 */
export const raycastStore: string | undefined = undefined; // "https://www.raycast.com/james_ko/mash"

/** Raycast's own red, for its mark on that button. */
export const raycastRed = "#ff6363";
