/**
 * Plays the welcome page as the reader scrolls, with GSAP.
 *
 * Each scene pins under the header and owns one paused timeline. Its drawing,
 * illustration, and captions take their state from the scene's position in
 * the viewport. Jumping or reversing the scroll seeks directly to that state.
 *
 * Without this script the page shows everything, statically.
 */
import { gsap } from "gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { draw, measure, parts, type Scene, sceneIn } from "./welcome-scene";

gsap.registerPlugin(ScrollTrigger, DrawSVGPlugin);
// A scene may have no drawing, and a drawing may have no filled shapes.
gsap.config({ nullTargetWarn: false });

const clamp = (x: number): number => Math.min(1, Math.max(0, x));

/** Eased progress of `x` from `from` to `to`. */
function smooth(from: number, to: number, x: number): number {
  const t = clamp((x - from) / (to - from));
  return t * t * (3 - 2 * t);
}

/** A drawing outside the scenes, drawn and undrawn by its own place on screen. */
interface Backdrop {
  readonly art: SVGSVGElement;
  readonly timeline: gsap.core.Timeline;
}

/** Builds the page's motion for the current layout; returns what removes it. */
function build(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
  let scenes: Scene[] = [];
  const backdrops: Backdrop[] = [];
  const reveals: { group: HTMLElement; timeline: gsap.core.Timeline }[] = [];

  function place(): void {
    const vh = innerHeight;

    for (const backdrop of backdrops) {
      // By the drawing's own place on screen, so a section taller than the
      // screen behaves like a short one: drawn as the drawing rises into
      // view, undrawn as it leaves by the top, and the reverse on the way up.
      const box = backdrop.art.getBoundingClientRect();
      const arriving = clamp((vh * 0.92 - box.top) / (vh * 0.45));
      const leaving = clamp((vh * 0.3 - box.bottom) / (vh * 0.25));
      backdrop.timeline.progress(arriving * (1 - leaving));
    }

    for (const { group, timeline } of reveals) {
      const entry = (vh - group.getBoundingClientRect().top) / vh;
      timeline.progress(smooth(0.04, 0.35, entry));
    }

    for (const scene of scenes) {
      const box = scene.el.getBoundingClientRect();
      const pinnedAt = Number.parseFloat(getComputedStyle(scene.stage).top) || 0;
      const tall = scene.stage.offsetHeight;
      const enter = clamp((vh - box.top) / vh);
      const exit = clamp((pinnedAt + tall - box.bottom) / tall);
      const visible = smooth(0.35, 0.95, enter) * (1 - smooth(0.1, 0.7, exit));
      scene.el.style.setProperty("--o", String(visible));
      // Draw as the section arrives, then scrub the rest over the sticky travel.
      const arrival = clamp((vh - box.top) / Math.max(1, vh - pinnedAt));
      const pinned = clamp((pinnedAt - box.top) / Math.max(1, box.height - tall));
      const time = scene.drawnAt * arrival + (scene.timeline.duration() - scene.drawnAt) * pinned;
      scene.timeline.time(time);
    }
  }

  let queued = 0;
  const request = (): void => {
    if (queued !== 0) return;
    queued = requestAnimationFrame(() => {
      queued = 0;
      place();
    });
  };

  const context = gsap.context(() => {
    scenes = [...root.querySelectorAll<HTMLElement>(".scene")].flatMap((el, index) => {
      const scene = sceneIn(el, index);
      return scene ? [scene] : [];
    });

    for (const section of root.querySelectorAll<HTMLElement>(".backdrop")) {
      const art = section.querySelector<SVGSVGElement>("svg.art");
      if (!art) continue;
      const strength = art.dataset.strength;
      if (strength) gsap.set(section, { "--art": strength });
      const timeline = gsap.timeline({ paused: true });
      const { strokes, dots } = parts(art);
      draw(timeline, strokes, dots);
      backdrops.push({ art, timeline });
    }

    // The hero stays whole until most of it has left the screen: the fade
    // runs over the last stretch, as its bottom edge nears the top.
    if (hero) {
      gsap.to(hero, {
        opacity: 0,
        y: -32,
        scale: 0.94,
        ease: "none",
        scrollTrigger: { trigger: hero, start: "bottom 40%", end: "bottom top", scrub: true },
      });
    }

    for (const group of root.querySelectorAll<HTMLElement>("[data-reveal]")) {
      const items = [...group.children];
      gsap.set(items, { opacity: 0, y: 32, scale: 0.97 });
      const timeline = gsap.timeline({ paused: true });
      timeline.to(items, {
        opacity: 1,
        y: 0,
        scale: 1,
        duration: 0.9,
        ease: "power3.out",
        stagger: 0.14,
      });
      reveals.push({ group, timeline });
    }
  }, root);

  for (const scene of scenes) measure(scene);
  place();
  addEventListener("scroll", request, { passive: true, signal: stop.signal });
  // Whenever a heading or panel changes size, fit it again: a web font that
  // arrives after the page was built makes a heading wider than it measured.
  const sized = new ResizeObserver((entries) => {
    for (const scene of scenes) {
      if (entries.some((entry) => scene.el.contains(entry.target))) measure(scene);
    }
    request();
  });
  for (const scene of scenes) {
    const heading = scene.head?.querySelector("h2") ?? scene.head;
    if (heading) sized.observe(heading);
    if (scene.copy && scene.copy !== heading) sized.observe(scene.copy);
    sized.observe(scene.show);
  }
  stop.signal.addEventListener("abort", () => sized.disconnect());
  // Small print opened inside a panel makes it taller: fit it again.
  root.addEventListener(
    "toggle",
    () => {
      for (const scene of scenes) measure(scene);
      request();
    },
    { capture: true, signal: stop.signal },
  );
  addEventListener(
    "resize",
    () => {
      for (const scene of scenes) measure(scene);
      request();
    },
    { signal: stop.signal },
  );

  return () => {
    stop.abort();
    if (queued !== 0) cancelAnimationFrame(queued);
    for (const scene of scenes) {
      scene.el.classList.remove("tight");
      for (const name of ["--o", "--fit", "--lift", "--head-travel", "--show-travel"])
        scene.el.style.removeProperty(name);
      scene.copy?.style.removeProperty("--grow");
    }
    context.revert();
  };
}

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement, hero: HTMLElement | null): () => void {
  // Give every scene its final height before switching to the pinned layout.
  const sections = root.querySelectorAll<HTMLElement>(".scene");
  for (const section of sections) {
    const captions = section.querySelectorAll(".caps > li");
    section.style.setProperty("--n", String(Math.max(1, captions.length)));
  }
  root.classList.add("motion");
  const unbuild = build(root, hero);
  return () => {
    unbuild();
    root.classList.remove("motion");
    for (const section of sections) section.style.removeProperty("--n");
  };
}
