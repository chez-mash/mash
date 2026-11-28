/**
 * Mash's tagline: the description of any page that gives none, and the
 * tagline in the landing page's title and link previews. astro.config.mjs
 * repeats it as Starlight's default description.
 */
export const tagline = "Supercharged context for your agents.";

/** The landing page's headline, one line each. */
export const headline = ["Your agents forget.", "Mash doesn’t."] as const;

/**
 * Where the landing page's Raycast button leads: the extension's source until
 * the Raycast Store lists it (raycast/extensions#33948), then its page there,
 * https://www.raycast.com/james_ko/mash.
 */
export const raycastLink = "https://github.com/mashhq/mash/tree/main/raycast";

/**
 * A darker tone of Raycast's red (#ff6363) and white on it, for that button:
 * white on Raycast's own red is 2.9 to 1, too faint to read; on this it is
 * 4.9 to 1.
 */
export const raycastRed = "#d13438";
export const raycastInk = "#ffffff";

/** Ferris's own orange, as rustacean.net draws him, for his outline on the landing page. */
export const ferrisOrange = "#f74c00";
