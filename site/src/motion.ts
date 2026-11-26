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

gsap.registerPlugin(ScrollTrigger);

/** What an empty quick search field says, as the desktop app's does. */
const placeholder = "Search every chat";

type Timeline = gsap.core.Timeline;
type Stop = () => void;

/** Which variant of each scene this screen gets. */
interface Screen {
  /** A wide screen with a mouse: longer rises, the two loops, the slide and the typing. */
  wide: boolean;
  /** Wide and tall enough to hold the quick search panel while it pins. */
  pins: boolean;
}

const all = <T extends Element = HTMLElement>(scope: ParentNode, selector: string): T[] => [
  ...scope.querySelectorAll<T>(selector),
];

/** Whether the element is still under the fold, so its entrance would be seen. */
const below = (element: Element): boolean => element.getBoundingClientRect().top >= innerHeight;

/** Fades targets up from `y` pixels lower, then hands them back to the stylesheet. */
function rise(
  timeline: Timeline,
  targets: gsap.TweenTarget,
  at: number,
  vars: { y: number; duration: number; ease?: string; stagger?: number },
): void {
  timeline.fromTo(
    targets,
    { opacity: 0, y: vars.y },
    {
      opacity: 1,
      y: 0,
      duration: vars.duration,
      ease: vars.ease ?? "power2.out",
      stagger: vars.stagger ?? 0,
      clearProps: "opacity,transform",
    },
    at,
  );
}

/** Plays a paused timeline once, when its trigger reaches `start`. */
function playOnce(trigger: Element, timeline: Timeline, start = "top 78%"): void {
  ScrollTrigger.create({ trigger, start, once: true, onEnter: () => timeline.play() });
}

/** Runs a loop only while its trigger is on screen. */
function loopWhileSeen(trigger: Element, loops: gsap.core.Tween[]): void {
  for (const loop of loops) loop.pause();
  ScrollTrigger.create({
    trigger,
    start: "top bottom",
    end: "bottom top",
    onToggle: (self) => {
      for (const loop of loops) loop.paused(!self.isActive);
    },
  });
}

/** A. The hero: the app icon settles, the headline rises a line at a time, the rest follows. */
function hero(root: HTMLElement, screen: Screen, entrance: boolean): void {
  if (entrance) {
    const y = screen.wide ? 24 : 16;
    const timeline = gsap.timeline();
    timeline.fromTo(
      all(root, ".hero-icon"),
      { opacity: 0, scale: 0.92 },
      { opacity: 1, scale: 1, duration: 0.6, ease: "power3.out", clearProps: "opacity,transform" },
      0,
    );
    timeline.fromTo(
      all(root, ".hero-icon .halo"),
      { opacity: 0, scale: 0.8 },
      { opacity: 1, scale: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity,transform" },
      0.15,
    );
    rise(timeline, all(root, "h1 span"), 0.12, {
      y,
      duration: 0.5,
      ease: "power3.out",
      stagger: 0.1,
    });
    rise(timeline, all(root, ".tagline"), 0.38, { y: 16, duration: 0.44 });
    rise(timeline, all(root, ".actions, .download-note"), 0.46, { y: 16, duration: 0.44 });
  }
  if (!screen.wide) return;
  // The streaks drift a few pixels for as long as the hero is on screen.
  const drift = all(root, ".streak").map((streak, index) =>
    gsap
      .fromTo(
        streak,
        { x: -14, y: 5 },
        { x: 14, y: -5, duration: 9 + index * 1.4, ease: "sine.inOut", yoyo: true, repeat: -1 },
      )
      .progress(0.5),
  );
  loopWhileSeen(root, drift);
}

/** E. The six agents come in one after another with a small lift. */
function agents(list: HTMLElement): void {
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

/**
 * B. Quick search: the query types itself, the results arrive, and the
 * selection moves down two rows. On a wide screen the panel pins and the
 * scroll drives it; elsewhere it plays once. The list keeps its height
 * throughout, so the panel never changes size.
 */
function quickSearch(band: HTMLElement, screen: Screen): Stop {
  const desk = band.querySelector<HTMLElement>(".desk");
  const panel = band.querySelector<HTMLElement>(".quick");
  const field = band.querySelector<HTMLElement>(".quick-q b");
  const rows = all(band, ".quick li");
  const [first, second] = rows;
  if (!desk || !panel || !field || !first || !second) return () => {};
  if (!screen.pins && !below(panel)) return () => {};

  const query = field.textContent ?? "";
  const typed = { count: 0 };
  const type = () => {
    const count = Math.round(typed.count);
    field.textContent = count === 0 ? placeholder : query.slice(0, count);
    field.classList.toggle("empty", count === 0);
  };
  // The selection is one highlight that slides between rows.
  const selection = document.createElement("i");
  selection.className = "quick-sel";
  selection.style.top = `${first.offsetTop}px`;
  const step = second.offsetTop - first.offsetTop;
  panel.prepend(selection);
  panel.classList.add("live");
  type();

  // Times are fractions of the pin when scrubbed and seconds when played.
  const at = screen.pins
    ? { type: 0, typing: 0.3, rows: 0.3, row: 0.08, apart: 0.03, down: [0.55, 0.7], move: 0.15 }
    : { type: 0.3, typing: 0.6, rows: 0.9, row: 0.2, apart: 0.06, down: [1.4, 1.65], move: 0.2 };
  const timeline = gsap.timeline({ paused: true });
  timeline.to(
    typed,
    { count: query.length, duration: at.typing, ease: `steps(${query.length})`, onUpdate: type },
    at.type,
  );
  timeline.fromTo(
    all(band, ".quick-h"),
    { opacity: 0 },
    { opacity: 1, duration: at.row, ease: "power2.out" },
    at.rows,
  );
  timeline.fromTo(
    rows,
    { opacity: 0, y: 8 },
    {
      opacity: 1,
      y: 0,
      duration: at.row,
      ease: "power2.out",
      stagger: at.apart,
      // A scrubbed row may be scrolled back, so only a played one is handed back.
      clearProps: screen.pins ? "" : "opacity,transform",
    },
    at.rows,
  );
  timeline.fromTo(
    selection,
    { opacity: 0 },
    { opacity: 1, duration: at.row, ease: "power2.out" },
    at.rows + at.apart,
  );
  timeline.to(selection, { y: step, duration: at.move, ease: "power2.out" }, at.down[0]);
  timeline.to(selection, { y: step * 2, duration: at.move, ease: "power2.out" }, at.down[1]);

  if (screen.pins) {
    // The last part of the pin holds the finished panel before it lets go.
    timeline.to({}, { duration: 0.15 }, 0.85);
    ScrollTrigger.create({
      trigger: desk,
      start: "top 12%",
      end: "+=150%",
      pin: true,
      // The wrapper is a flex column, where the spacing is off unless asked for.
      pinSpacing: true,
      // Measured before the reveals below it, whose places depend on this spacing.
      refreshPriority: 1,
      scrub: 0.4,
      animation: timeline,
    });
  } else {
    timeline.fromTo(
      panel,
      { opacity: 0, y: 16 },
      { opacity: 1, y: 0, duration: 0.4, ease: "power2.out", clearProps: "opacity,transform" },
      0,
    );
    playOnce(panel, timeline, "top 70%");
  }

  return () => {
    selection.remove();
    panel.classList.remove("live");
    field.classList.remove("empty");
    field.textContent = query;
  };
}

/** C. Search: the results arrive, then the highlight sweeps across each match. */
function search(band: HTMLElement, screen: Screen): void {
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
function memory(band: HTMLElement, screen: Screen): void {
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
  // The "saved" line draws: its arrow by its stroke, its words by a wipe.
  for (const stroke of all<SVGGeometryElement>(saved, "svg path, svg line, svg polyline")) {
    const length = stroke.getTotalLength();
    timeline.fromTo(
      stroke,
      { strokeDasharray: length, strokeDashoffset: length },
      {
        strokeDashoffset: 0,
        duration: 0.4,
        ease: "power2.out",
        clearProps: "strokeDasharray,strokeDashoffset",
      },
      0.45,
    );
  }
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
function stats(list: HTMLElement): Stop {
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
 * G. Mashy: his first answer types out in place and its quote follows, and on
 * a wide screen he bobs. The answer's whole text holds its space while it
 * types, so the window keeps its size. His other examples are left alone.
 */
function mashy(band: HTMLElement, screen: Screen): Stop {
  const face = band.querySelector<HTMLElement>(".mashy-face");
  if (face && screen.wide) {
    const bob = gsap
      .fromTo(face, { y: -2 }, { y: 2, duration: 1.5, ease: "sine.inOut", yoyo: true, repeat: -1 })
      .progress(0.5);
    loopWhileSeen(face, [bob]);
  }
  const picture = band.querySelector<HTMLElement>(".mashy-window");
  const example = band.querySelector<HTMLElement>('.example[data-example="1"]');
  const answer = example?.querySelector<HTMLElement>(".answer");
  const sentence = answer?.querySelector<HTMLElement>(":scope > div > p");
  if (!picture || !answer || !sentence || !below(picture)) return () => {};

  const timeline = gsap.timeline({ paused: true });
  if (!screen.wide) {
    rise(timeline, answer, 0, { y: 12, duration: 0.45 });
    playOnce(picture, timeline);
    return () => {};
  }
  const text = sentence.textContent ?? "";
  const said = document.createElement("span");
  const unsaid = document.createElement("span");
  unsaid.style.opacity = "0";
  unsaid.textContent = text;
  sentence.replaceChildren(said, unsaid);
  const typed = { count: 0 };
  const restore = () => sentence.replaceChildren(text);
  timeline.fromTo(
    all(answer, ":scope > img"),
    { opacity: 0 },
    { opacity: 1, duration: 0.2, ease: "power2.out", clearProps: "opacity" },
    0,
  );
  timeline.to(
    typed,
    {
      count: text.length,
      duration: 0.9,
      ease: "none",
      onUpdate: () => {
        const count = Math.round(typed.count);
        said.textContent = text.slice(0, count);
        unsaid.textContent = text.slice(count);
      },
    },
    0.15,
  );
  rise(timeline, all(answer, ".quote"), 1.05, { y: 8, duration: 0.35 });
  timeline.eventCallback("onComplete", restore);
  playOnce(picture, timeline);
  return restore;
}

/** H. The closing card: the app icon rises with the hero's glow. */
function closing(card: HTMLElement, screen: Screen): void {
  if (!below(card)) return;
  const timeline = gsap.timeline({ paused: true });
  timeline.fromTo(
    all(card, ".close-icon"),
    { opacity: 0, y: screen.wide ? 28 : 16, scale: 0.92 },
    {
      opacity: 1,
      y: 0,
      scale: 1,
      duration: 0.7,
      ease: "power3.out",
      clearProps: "opacity,transform",
    },
    0,
  );
  timeline.fromTo(
    all(card, ".close-icon .halo"),
    { opacity: 0, scale: 0.8 },
    { opacity: 1, scale: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity,transform" },
    0.15,
  );
  timeline.fromTo(
    all(card, ".close-glow"),
    { opacity: 0 },
    { opacity: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity" },
    0.15,
  );
  rise(timeline, all(card, "h2"), 0.2, { y: 16, duration: 0.5, ease: "power3.out" });
  rise(timeline, all(card, ".actions"), 0.32, { y: 16, duration: 0.5 });
  playOnce(card, timeline, "top 82%");
}

/** Every other band fades up as it scrolls into view. `shown` marks a band that has arrived. */
function bands(welcome: HTMLElement, screen: Screen): void {
  for (const band of all(welcome, "[data-reveal]")) {
    // These bring their own parts in.
    const staged = band.matches(".closing, .mash-agents ul");
    // A transform on the pinned panel's wrapper would unpin it, so its parts fade separately.
    const parts =
      screen.pins && band.matches(".shortcuts")
        ? { rise: all(band, "h2"), fade: all(band, ".desk") }
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

/**
 * The pinned panel adds its scroll distance to the page once the script runs.
 * A page opened at a link to something under it would be left short of it, so
 * go to the link's target again.
 */
function keepAnchor(pinned: Element | null): void {
  if (!pinned || location.hash.length < 2) return;
  const target = document.getElementById(decodeURIComponent(location.hash.slice(1)));
  if (!target) return;
  if (pinned.compareDocumentPosition(target) & Node.DOCUMENT_POSITION_FOLLOWING) {
    ScrollTrigger.refresh();
    target.scrollIntoView();
  }
}

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
