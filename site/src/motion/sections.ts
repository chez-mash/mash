/**
 * The scenes that play once as their band scrolls into view: the agents, the
 * search results, the saved decision, the stats and Ferris, and the plain
 * fade every other band gets.
 */
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { all, below, playOnce, rise, type Screen, type Stop } from "./stage";

/** E. The six agents come in one after another with a small lift. */
export function agents(list: HTMLElement): void {
  if (!below(list)) return;
  const timeline = gsap.timeline({ paused: true });
  rise(timeline, all(list, "li"), 0, { y: 10, duration: 0.42, stagger: 0.04 });
  timeline.fromTo(
    all(list, ".tile"),
    { scale: 0.86 },
    { scale: 1, duration: 0.42, ease: "power2.out", stagger: 0.04, clearProps: "transform" },
    0,
  );
  playOnce(list, timeline, "top 85%");
}

/** C. Search: the results arrive, then the highlight sweeps across each match. */
export function search(band: HTMLElement, screen: Screen): void {
  const picture = band.querySelector<HTMLElement>(".search-window");
  if (!picture || !below(picture)) return;
  const marks = all(picture, "mark");
  const swept = "backgroundImage,backgroundRepeat,backgroundColor,backgroundSize,boxShadow";
  gsap.set(marks, {
    backgroundImage: "linear-gradient(var(--find-bg), var(--find-bg))",
    backgroundRepeat: "no-repeat",
    backgroundColor: "transparent",
    backgroundSize: "0% 100%",
    boxShadow: "none",
  });
  const timeline = gsap.timeline({ paused: true });
  rise(timeline, all(picture, ".result"), 0, {
    y: screen.wide ? 14 : 10,
    duration: 0.45,
    ease: "power3.out",
    stagger: 0.09,
  });
  timeline.to(
    marks,
    {
      backgroundSize: "100% 100%",
      duration: 0.35,
      ease: "power2.out",
      stagger: 0.08,
      clearProps: swept,
    },
    0.5,
  );
  playOnce(picture, timeline);
}

/** D. Memory: the saved decision comes out from under the message it was saved from. */
export function memory(band: HTMLElement, screen: Screen): void {
  const thread = band.querySelector<HTMLElement>(".memory-thread");
  const said = thread?.querySelector<HTMLElement>(".bubble.said");
  const record = thread?.querySelector<HTMLElement>(".record");
  const [saved, returns] = thread ? all(thread, ".step") : [];
  if (!thread || !said || !record || !saved || !returns || !below(thread)) return;

  const timeline = gsap.timeline({ paused: true });
  if (screen.wide) {
    gsap.set(said, { position: "relative", zIndex: 2 });
    timeline.fromTo(
      record,
      { y: -64, scale: 0.97, transformOrigin: "85% 0%" },
      {
        y: 0,
        scale: 1,
        duration: 0.7,
        ease: "power3.out",
        clearProps: "transform,transformOrigin",
      },
      0.1,
    );
    timeline.fromTo(
      record,
      { opacity: 0 },
      { opacity: 1, duration: 0.28, ease: "none", clearProps: "opacity" },
      0.1,
    );
    timeline.set(said, { clearProps: "position,zIndex" }, 0.8);
  } else {
    rise(timeline, record, 0.1, { y: 16, duration: 0.7, ease: "power3.out" });
  }
  // The "saved" line draws by a wipe: its arrow, then its words.
  timeline.fromTo(
    saved,
    { clipPath: "inset(0% 100% 0% 0%)" },
    { clipPath: "inset(0% 0% 0% 0%)", duration: 0.4, ease: "power2.out", clearProps: "clipPath" },
    0.45,
  );
  rise(timeline, returns, 0.8, { y: 10, duration: 0.35 });
  rise(timeline, all(thread, ".later"), 0.9, { y: 10, duration: 0.35 });
  playOnce(thread, timeline);
}

/**
 * F. Stats: the numbers count up once. Each number keeps its final width and
 * fills it from the unit's side, so the unit beside it never moves.
 */
export function stats(list: HTMLElement): Stop {
  if (!below(list)) return () => {};
  const numbers = all(list, "dt b").map((node) => ({ node, text: node.textContent ?? "" }));
  const timeline = gsap.timeline({ paused: true });
  rise(timeline, all(list, ":scope > div"), 0, { y: 12, duration: 0.4, stagger: 0.06 });
  for (const { node, text } of numbers) {
    const count = { value: 0 };
    timeline.to(
      count,
      {
        value: Number(text),
        duration: 0.8,
        ease: "power2.out",
        onUpdate: () => {
          node.textContent = String(Math.round(count.value));
        },
      },
      0.1,
    );
  }
  const restore = () => {
    for (const { node, text } of numbers) {
      node.textContent = text;
      node.style.removeProperty("display");
      node.style.removeProperty("min-width");
      node.style.removeProperty("text-align");
    }
  };
  timeline.eventCallback("onComplete", restore);
  ScrollTrigger.create({
    trigger: list,
    start: "top 78%",
    once: true,
    onEnter: () => {
      for (const { node } of numbers) {
        node.style.display = "inline-block";
        node.style.minWidth = `${node.getBoundingClientRect().width}px`;
        node.style.textAlign = "end";
        node.textContent = "0";
      }
      timeline.play();
    },
  });
  return restore;
}

/**
 * G. Ferris draws himself: the shell first, then each claw and leg, a stroke
 * at a time, and last the eyes and the smile. Every stroke is one unit long
 * (`pathLength="1"`), so one dash hides it and the dash's offset draws it.
 * Until its turn a stroke is unseen, because a round cap shows as a dot at
 * the start of even a wholly offset dash.
 */
export function ferris(drawing: SVGElement): void {
  if (!below(drawing)) return;
  const strokes = all<SVGPathElement>(drawing, "path[pathLength]");
  const solids = all<SVGPathElement>(drawing, ".dot");
  const timeline = gsap.timeline({ paused: true });
  gsap.set(strokes, { strokeDasharray: 1, strokeDashoffset: 1, opacity: 0 });
  gsap.set(solids, { opacity: 0 });
  timeline.set(strokes, { opacity: 1, stagger: 0.14 }, 0.15);
  timeline.to(
    strokes,
    {
      strokeDashoffset: 0,
      duration: 0.9,
      ease: "power2.inOut",
      stagger: 0.14,
      clearProps: "strokeDasharray,strokeDashoffset,opacity",
    },
    0.15,
  );
  timeline.to(
    solids,
    { opacity: 1, duration: 0.3, ease: "power2.out", stagger: 0.06, clearProps: "opacity" },
    0.15 + 0.9 + 0.14 * (strokes.length - 1) - 0.2,
  );
  playOnce(drawing, timeline, "top 85%");
}

/** Every other band fades up as it scrolls into view. `shown` marks a band that has arrived. */
export function bands(welcome: HTMLElement, screen: Screen): void {
  for (const band of all(welcome, "[data-reveal]")) {
    // These bring their own parts in.
    const staged = band.matches(".closing, .mash-agents ul");
    // A transform on the pinned panel's wrapper would unpin it, so its parts fade separately.
    const parts =
      screen.pins && band.matches(".shortcuts")
        ? { rise: all(band, ".head"), fade: all(band, ".desk") }
        : null;
    if (staged || !below(band)) {
      band.classList.add("shown");
      continue;
    }
    const timeline = gsap.timeline({ paused: true });
    if (parts) {
      rise(timeline, parts.rise, 0, { y: 24, duration: 0.6 });
      timeline.fromTo(
        parts.fade,
        { opacity: 0 },
        { opacity: 1, duration: 0.6, ease: "power2.out", clearProps: "opacity" },
        0,
      );
    } else {
      rise(timeline, band, 0, { y: 24, duration: 0.6 });
    }
    ScrollTrigger.create({
      trigger: band,
      start: "top 92%",
      once: true,
      onEnter: () => {
        band.classList.add("shown");
        timeline.play();
      },
    });
  }
}
