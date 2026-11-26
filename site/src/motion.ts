/**
 * The landing page's motion: eight small scenes built around the product
 * windows, as the motion storyboard draws them.
 *
 * The page is complete as served. This script sets each scene's starting
 * state and plays it once as the scene scrolls into view; a scene that is
 * already on screen when the script starts is left as it is. Only two things
 * loop, the hero's butter streaks and Mashy's bob, and both stop off screen.
 * One scene pins: the quick search panel, on a wide screen with a mouse, where
 * scrolling types the search and moves the selection. Everywhere else that
 * panel plays once. Scenes move transform, opacity, clip-path and
 * background-size only, so nothing on the page changes place.
 */
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { closing, hero } from "./motion/hero";
import { mashy } from "./motion/mashy";
import { keepAnchor, quickSearch } from "./motion/quickSearch";
import { agents, bands, memory, search, stats } from "./motion/sections";
import { all, type Screen, type Stop } from "./motion/stage";

gsap.registerPlugin(ScrollTrigger);

/** Starts the landing page's motion; returns what stops it and restores the page. */
export function mountMotion(page: ParentNode): Stop {
  const top = page.querySelector<HTMLElement>(".mash-hero");
  const welcome = page.querySelector<HTMLElement>(".mash-welcome");
  let entered = false;
  const media = gsap.matchMedia();
  media.add(
    {
      every: "all",
      wide: "(min-width: 64rem) and (pointer: fine)",
      tall: "(min-height: 47.5rem)",
    },
    (context) => {
      const conditions = context.conditions ?? {};
      const wide = Boolean(conditions.wide);
      const screen: Screen = { wide, pins: wide && Boolean(conditions.tall) };
      const stops: Stop[] = [];
      // The hero's entrance plays once, however the window is resized afterwards.
      if (top) hero(top, screen, !entered);
      entered = true;
      if (top) delete top.dataset.motion;
      if (welcome) {
        const band = (id: string) => welcome.querySelector<HTMLElement>(`#${id}`);
        const list = welcome.querySelector<HTMLElement>(".mash-agents ul");
        if (list) agents(list);
        bands(welcome, screen);
        const shortcuts = band("welcome-shortcuts");
        if (shortcuts) stops.push(quickSearch(shortcuts, screen));
        const searching = band("welcome-search");
        if (searching) search(searching, screen);
        const asking = band("welcome-mashy");
        if (asking) stops.push(mashy(asking, screen));
        const remembering = band("welcome-memory");
        if (remembering) memory(remembering, screen);
        const figures = welcome.querySelector<HTMLElement>(".stats");
        if (figures) stops.push(stats(figures));
        const card = welcome.querySelector<HTMLElement>(".closing");
        if (card) closing(card, screen);
        if (screen.pins) keepAnchor(shortcuts?.querySelector(".desk") ?? null);
      }
      return () => {
        for (const stop of stops) stop();
        if (welcome)
          for (const band of all(welcome, "[data-reveal]")) band.classList.remove("shown");
      };
    },
  );
  return () => media.revert();
}
