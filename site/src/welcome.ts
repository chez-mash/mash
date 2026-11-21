/**
 * Plays the welcome page as the reader scrolls, with GSAP.
 *
 * Each scene pins under the header and owns one timeline: its drawing is
 * drawn, its heading makes way for an illustration, the captions arrive one
 * after another, then the illustration leaves and the drawing is undrawn. The scroll does
 * not scrub that timeline. It only says how far the scene should have got, and
 * the timeline plays there at its own pace, forwards or backwards, so a fast
 * scroll cannot skip anything.
 *
 * A scene scrolled past before it has finished is kept fixed on screen until
 * it has, and the scenes after it wait their turn. Nothing here ever moves or
 * refuses the page's own scroll: Safari ignores that mid-gesture and shakes.
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

/** How much faster than written a timeline runs backwards, and when kept. */
const REWIND = 2.2;
const KEPT_PACE = 1.6;

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
  shown: number;
}

/** Builds the page's motion for the current layout; returns what removes it. */
function build(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
  let scenes: Scene[] = [];
  const backdrops: Backdrop[] = [];
  /** Items outside the scenes that have not come into view yet. */
  let waiting: HTMLElement[] = [];
  /** The scene being kept on screen to finish, if any. */
  let keeper: Scene | null = null;
  /** Scenes passed by the reader that still have to finish in order. */
  const pending = new Set<Scene>();
  let previousScroll = scrollY;

  const unkeep = (scene: Scene): void => {
    scene.kept = false;
    scene.el.classList.remove("kept");
    scene.stage.style.left = "";
    scene.stage.style.width = "";
    if (keeper === scene) keeper = null;
  };

  /** Plays a scene's timeline to a time, at its own pace. */
  const send = (scene: Scene, time: number): void => {
    if (Math.abs(time - scene.target) < 0.001) return;
    scene.target = time;
    const from = scene.timeline.time();
    scene.timeline.tweenTo(time, {
      duration: Math.abs(time - from) / (time < from ? REWIND : 1),
      ease: "none",
      overwrite: true,
    });
  };

  /** Puts a scene's timeline at a time at once: for a scene nobody is watching. */
  const put = (scene: Scene, time: number): void => {
    gsap.killTweensOf(scene.timeline);
    scene.target = time;
    if (scene.timeline.time() !== time) scene.timeline.time(time);
  };

  /** Keeps a scene where it was pinned until it has played out, then fades it. */
  const keep = (scene: Scene, box: DOMRect): void => {
    scene.kept = true;
    keeper = scene;
    scene.el.classList.add("kept");
    scene.stage.style.left = `${box.left}px`;
    scene.stage.style.width = `${box.width}px`;
    scene.el.style.setProperty("--o", "1");
    for (const later of scenes) if (later.index > scene.index) put(later, 0);
    scene.target = scene.endAt;
    scene.timeline.tweenTo(scene.endAt, {
      duration: (scene.endAt - scene.timeline.time()) / KEPT_PACE,
      ease: "none",
      overwrite: true,
      onComplete: () => {
        if (!scene.kept) return;
        gsap.to(scene.el, {
          "--o": 0,
          duration: 0.28,
          ease: "none",
          onComplete: () => {
            unkeep(scene);
            place(false);
          },
        });
      },
    });
  };

  /**
   * Reads the scroll and says where everything should be. `settled` puts it
   * there at once, for a page just loaded or laid out afresh.
   */
  function place(settled: boolean): void {
    const vh = innerHeight;
    const current = scrollY;
    const previous = previousScroll;
    previousScroll = current;

    for (const backdrop of backdrops) {
      // By the drawing's own place on screen, so a section taller than the
      // screen behaves like a short one: drawn as the drawing rises into
      // view, undrawn as it leaves by the top, and the reverse on the way up.
      const box = backdrop.art.getBoundingClientRect();
      const arriving = clamp((vh * 0.92 - box.top) / (vh * 0.45));
      const leaving = clamp((vh * 0.3 - box.bottom) / (vh * 0.25));
      const wanted = arriving * (1 - leaving);
      if (!settled && Math.abs(wanted - backdrop.shown) < 0.001) continue;
      backdrop.shown = wanted;
      gsap.to(backdrop.timeline, {
        progress: wanted,
        duration: settled ? 0 : 0.35,
        ease: "power1.out",
        overwrite: true,
      });
    }

    // The agent marks and closing text rise in once their part of the page
    // is visible and the reader can see past any kept scene.
    if (!keeper && waiting.length > 0) {
      const arrived = waiting.filter((item) => {
        const box = item.getBoundingClientRect();
        return box.top < vh * 0.98 && box.bottom > 0;
      });
      waiting = waiting.filter((item) => !arrived.includes(item));
      arrived.forEach((item, i) => {
        item.classList.add("seen");
        const delay = i * 0.14;
        gsap.to(item, { autoAlpha: 1, y: 0, scale: 1, duration: 0.9, ease: "power3.out", delay });
      });
    }

    for (const scene of scenes) {
      const box = scene.el.getBoundingClientRect();
      const pinnedAt = Number.parseFloat(getComputedStyle(scene.stage).top) || 0;
      const tall = scene.stage.offsetHeight;
      const pinEnds = current + box.bottom - pinnedAt - tall + 1;
      const past = current > pinEnds;

      // Track crossings even while an earlier scene covers this one. A scene
      // can go from ahead of the viewport to past its pin in one frame.
      if (settled || !past) pending.delete(scene);
      else if (previous <= pinEnds) pending.add(scene);
      if (keeper && scene.index > keeper.index) continue;

      if (scene.kept) {
        // However far past the page has gone, it stays until it has played out.
        if (past) continue;
        // Scrolled back into it.
        unkeep(scene);
      }

      const enter = clamp((vh - box.top) / vh);
      const exit = clamp((pinnedAt + tall - box.bottom) / tall);
      const visible = smooth(0.35, 0.95, enter) * (1 - smooth(0.1, 0.7, exit));
      scene.el.style.setProperty("--o", String(visible));
      const onScreen = box.bottom > 0 && box.top < vh;

      const go = settled ? put : send;
      if (past) {
        const unfinished = scene.timeline.time() < scene.endAt - 0.02;
        const crossed = pending.delete(scene);
        if (crossed && unfinished && !keeper) keep(scene, box);
        else if (onScreen) go(scene, scene.endAt);
        else put(scene, scene.endAt);
        continue;
      }
      if (!onScreen) {
        // Not reached yet: at its very beginning.
        put(scene, 0);
        continue;
      }
      const last = scene.stepAt[scene.steps - 1] ?? scene.panelAt;
      if (!scene.head) {
        // Nothing to draw or title: its panel arrives as it nears its place,
        // stays while it is pinned, and leaves in the last stretch.
        const pinned = clamp((pinnedAt - box.top) / (box.height - tall));
        go(scene, pinned > 0.8 ? scene.endAt : enter > 0.7 ? last : 0);
        continue;
      }
      if (box.top > pinnedAt) {
        // Rising into place: the drawing is drawn as it comes.
        go(scene, scene.drawnAt * clamp((enter - 0.25) / 0.75));
        continue;
      }
      // Pinned: a part and a half for the heading alone, one per step, and a
      // last one to leave by.
      const raw = clamp((pinnedAt - box.top) / (box.height - tall)) * (scene.steps + 2.5);
      const position = raw < 1.5 ? raw / 1.5 : raw - 0.5;
      let time = scene.titledAt;
      if (position >= scene.steps + 1.2) time = scene.endAt;
      else if (position >= 0.45) {
        const reached = Math.min(scene.steps, Math.floor(position - 0.8) + 1);
        time = scene.stepAt[reached - 1] ?? scene.panelAt;
      }
      go(scene, time);
    }
  }

  let queued = 0;
  const request = (): void => {
    if (queued !== 0) return;
    queued = requestAnimationFrame(() => {
      queued = 0;
      place(false);
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
      if (strength) section.style.setProperty("--art", strength);
      const timeline = gsap.timeline({ paused: true });
      const { strokes, dots } = parts(art);
      draw(timeline, strokes, dots);
      backdrops.push({ art, timeline, shown: 0 });
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

    waiting = [...root.querySelectorAll<HTMLElement>("[data-reveal] > *")];
    gsap.set(waiting, { autoAlpha: 0, y: 32, scale: 0.97 });
  }, root);

  for (const scene of scenes) measure(scene);
  place(true);
  addEventListener("scroll", request, { passive: true, signal: stop.signal });
  // Whenever a heading or panel changes size, fit it again: a web font that
  // arrives after the page was built makes a heading wider than it measured.
  const sized = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const scene = scenes.find((one) => one.el.contains(entry.target));
      if (scene) measure(scene);
    }
  });
  for (const scene of scenes) {
    const heading = scene.head?.querySelector("h2") ?? scene.head;
    if (heading) sized.observe(heading);
    sized.observe(scene.show);
  }
  stop.signal.addEventListener("abort", () => sized.disconnect());
  // Small print opened inside a panel makes it taller: fit it again.
  root.addEventListener(
    "toggle",
    () => {
      for (const scene of scenes) measure(scene);
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
      gsap.killTweensOf([scene.timeline, scene.el]);
      unkeep(scene);
      scene.el.classList.remove("tight");
      for (const name of ["--o", "--fit", "--lift"]) scene.el.style.removeProperty(name);
      scene.head?.style.removeProperty("--grow");
    }
    for (const backdrop of backdrops) gsap.killTweensOf(backdrop.timeline);
    context.revert();
    for (const item of root.querySelectorAll(".seen")) item.classList.remove("seen");
  };
}

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
  // Give every scene its final height before switching to the pinned layout.
  // Keep the heights through font rebuilds so the browser can retain its anchor.
  const sections = root.querySelectorAll<HTMLElement>(".scene");
  for (const section of sections) {
    const captions = section.querySelectorAll(".caps > li");
    section.style.setProperty("--n", String(Math.max(1, captions.length)));
  }
  root.classList.add("motion");
  let unbuild = build(root, hero);
  // Heights are measured into the timelines, so a new layout needs new ones.
  // A phone's address bar changes only the height: that is not a new layout.
  let wide = innerWidth;
  const rebuild = (): void => {
    unbuild();
    unbuild = build(root, hero);
  };
  addEventListener(
    "resize",
    () => {
      if (innerWidth === wide) return;
      wide = innerWidth;
      rebuild();
    },
    { signal: stop.signal },
  );
  // Fonts are fetched when first used, which can be after this runs.
  document.fonts.addEventListener("loadingdone", rebuild, { signal: stop.signal });
  return () => {
    stop.abort();
    unbuild();
    root.classList.remove("motion");
    for (const section of sections) section.style.removeProperty("--n");
  };
}
