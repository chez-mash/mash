/**
 * Plays the welcome page as the reader scrolls, with GSAP.
 *
 * Each scene pins under the header and owns one timeline: its drawing is
 * drawn, its heading makes way for a panel, the panel's steps arrive one after
 * another, then the panel leaves and the drawing is undrawn. The scroll does
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

gsap.registerPlugin(ScrollTrigger, DrawSVGPlugin);
// A drawing may have no filled shapes and a tile no icon: empty targets are expected.
gsap.config({ nullTargetWarn: false });

/** How much later each stroke of a drawing starts than the one before, in strokes. */
const STAGGER = 0.12;
/** Seconds a whole drawing takes to draw. */
const DRAW_SECONDS = 1;
/** How long a finished drawing has the screen to itself before its panel rises. */
const HOLD_SECONDS = 0.5;
/** Letters a second for a typed query. */
const TYPING_RATE = 22;
/** How much faster than written a timeline runs backwards, and when kept. */
const REWIND = 2.2;
const KEPT_PACE = 1.6;
/** How faint a drawing is behind its panel. */
const FAINT = 0.13;

const clamp = (x: number): number => Math.min(1, Math.max(0, x));

/** Eased progress of `x` from `from` to `to`. */
function smooth(from: number, to: number, x: number): number {
  const t = clamp((x - from) / (to - from));
  return t * t * (3 - 2 * t);
}

interface Scene {
  readonly index: number;
  readonly el: HTMLElement;
  readonly stage: HTMLElement;
  readonly head: HTMLElement;
  readonly show: HTMLElement;
  readonly art: SVGSVGElement | null;
  readonly steps: number;
  readonly timeline: gsap.core.Timeline;
  /** Timeline times: drawing done, heading alone, panel up, each step shown, the end. */
  readonly drawnAt: number;
  readonly titledAt: number;
  readonly panelAt: number;
  readonly stepAt: readonly number[];
  readonly endAt: number;
  /** Where the timeline was last sent. */
  target: number;
  /** Whether the reader has had the scene on screen during this pass. */
  entered: boolean;
  /** Whether it is fixed on screen after its place, finishing. */
  kept: boolean;
}

/** A drawing outside the scenes, drawn and undrawn by its own place on screen. */
interface Backdrop {
  readonly art: SVGSVGElement;
  readonly timeline: gsap.core.Timeline;
  shown: number;
}

/** The strokes of a drawing, and the shapes that fill in once they are drawn. */
function parts(art: Element | null): { strokes: SVGElement[]; dots: SVGElement[] } {
  if (!art) return { strokes: [], dots: [] };
  return {
    strokes: [...art.querySelectorAll<SVGElement>("path:not(.dot, .glint)")],
    dots: [...art.querySelectorAll<SVGElement>("circle, .dot, .glint")],
  };
}

/** Appends drawing the strokes one after another, then the filled shapes. */
function draw(timeline: gsap.core.Timeline, strokes: SVGElement[], dots: SVGElement[]): void {
  gsap.set(strokes, { drawSVG: "0%", autoAlpha: 0 });
  gsap.set(dots, { autoAlpha: 0 });
  const at = timeline.duration();
  const one = DRAW_SECONDS / (1 + Math.max(0, strokes.length - 1) * STAGGER);
  strokes.forEach((stroke, i) => {
    // Shown only once it has begun, so a round cap never sits there as a dot.
    timeline.to(stroke, { autoAlpha: 1, duration: 0.04, ease: "none" }, at + i * one * STAGGER);
    timeline.to(stroke, { drawSVG: "100%", duration: one, ease: "none" }, "<");
  });
  if (dots.length > 0) timeline.to(dots, { autoAlpha: 1, duration: 0.2, ease: "none" });
}

/** Appends undrawing them, last stroke first. */
function undraw(timeline: gsap.core.Timeline, strokes: SVGElement[], dots: SVGElement[]): void {
  if (dots.length > 0) timeline.to(dots, { autoAlpha: 0, duration: 0.12, ease: "none" });
  const at = timeline.duration();
  const one = 0.7 / (1 + Math.max(0, strokes.length - 1) * STAGGER);
  [...strokes].reverse().forEach((stroke, i) => {
    timeline.to(stroke, { drawSVG: "0%", duration: one, ease: "none" }, at + i * one * STAGGER);
    timeline.to(stroke, { autoAlpha: 0, duration: 0.04, ease: "none" }, ">-0.04");
  });
}

/** Builds one scene's whole performance as a paused timeline. */
function sceneIn(el: HTMLElement, index: number): Scene | undefined {
  const stage = el.querySelector<HTMLElement>(".stage");
  const head = el.querySelector<HTMLElement>(".head");
  const show = el.querySelector<HTMLElement>(".show");
  const steps = Number(el.dataset.steps);
  if (!stage || !head || !show || !Number.isFinite(steps) || steps < 1) return undefined;
  el.style.setProperty("--n", String(steps));
  const art = el.querySelector<SVGSVGElement>("svg.art");
  const strength = Number(art?.dataset.strength ?? 0.5);
  const { strokes, dots } = parts(art);
  const stepped = [...el.querySelectorAll<HTMLElement>("[data-step]")];
  const inThread = (item: HTMLElement): boolean =>
    item.parentElement?.classList.contains("grow") === true;
  const timeline = gsap.timeline({ paused: true });

  // The drawing, then a moment with the heading alone.
  gsap.set(el, { "--t": 0, "--art": strength });
  draw(timeline, strokes, dots);
  const drawnAt = timeline.duration();
  timeline.to({}, { duration: HOLD_SECONDS });
  const titledAt = timeline.duration();

  // The heading makes way for the panel, and the drawing recedes behind it.
  timeline.to(el, { "--t": 1, "--art": FAINT, duration: 0.55, ease: "power2.inOut" });
  const typed = el.querySelector<HTMLElement>("[data-type]");
  const query = typed?.textContent ?? "";
  const typing = { letters: 0 };
  if (typed) {
    timeline.to(typing, {
      letters: query.length,
      duration: query.length / TYPING_RATE,
      ease: "none",
    });
  }
  const panelAt = timeline.duration();

  for (const item of stepped) {
    gsap.set(
      item,
      inThread(item)
        ? { height: 0, marginTop: 0, autoAlpha: 0, y: 14 }
        : { autoAlpha: 0, y: 26, scale: 0.97 },
    );
  }

  // The steps. In a thread each turn takes its room as it arrives, pushing
  // older ones up; elsewhere a step joins the ones before it or, where they
  // share one place, takes over from the last.
  const stepAt: number[] = [];
  const picks: { readonly pick: HTMLElement; readonly at: number }[] = [];
  for (let step = 0; step < steps; step++) {
    let at = timeline.duration();
    // Suggestions: the one chosen lights up, then they fold away.
    const folding = stepped.filter((item) => Number(item.dataset.until) === step);
    for (const item of folding) {
      const pick = item.querySelector<HTMLElement>(".pick");
      if (pick) picks.push({ pick, at });
      timeline.to(
        item,
        { height: 0, marginTop: 0, autoAlpha: 0, duration: 0.35, ease: "power2.inOut" },
        at + 0.45,
      );
    }
    if (folding.length > 0) at += 0.7;
    for (const item of stepped) {
      if (Number(item.dataset.step) !== step) continue;
      if (inThread(item)) {
        timeline.to(
          item,
          { height: "auto", marginTop: 12, autoAlpha: 1, y: 0, duration: 0.5, ease: "power2.out" },
          at,
        );
        continue;
      }
      const parent = item.parentElement;
      const before = parent?.matches(".caps, .swap")
        ? stepped.filter(
            (other) => other.parentElement === parent && Number(other.dataset.step) === step - 1,
          )
        : [];
      if (before.length > 0) {
        timeline.to(
          before,
          { autoAlpha: 0, y: -26, scale: 0.97, duration: 0.3, ease: "power2.in" },
          at,
        );
      }
      timeline.to(
        item,
        { autoAlpha: 1, y: 0, scale: 1, duration: 0.45, ease: "power2.out" },
        at + (before.length > 0 ? 0.2 : 0),
      );
    }
    stepAt.push(timeline.duration());
  }

  // The panel leaves, the drawing comes back to strength and is undrawn.
  timeline.to(el, { "--t": 0, "--art": strength, duration: 0.45, ease: "power2.inOut" });
  undraw(timeline, strokes, dots);
  const endAt = timeline.duration();

  // What follows the timeline without being tweened: the letters typed so
  // far, and which suggestion has been picked.
  const follow = (): void => {
    if (typed) {
      const count = Math.floor(typing.letters + 0.001);
      typed.textContent = query.slice(0, count);
      typed.classList.toggle("caret", count < query.length);
    }
    const now = timeline.time();
    for (const { pick, at } of picks) pick.classList.toggle("picked", now >= at);
  };
  timeline.eventCallback("onUpdate", follow);
  follow();

  return {
    index,
    el,
    stage,
    head,
    show,
    art,
    steps,
    timeline,
    drawnAt,
    titledAt,
    panelAt,
    stepAt,
    endAt,
    target: 0,
    entered: false,
    kept: false,
  };
}

/**
 * Sizes a scene for the screen: a panel too tall is scaled down to fit, and the
 * heading learns how far down it sits while it has the screen to itself.
 */
function measure(scene: Scene): void {
  const gap = Number.parseFloat(getComputedStyle(scene.stage).rowGap) || 0;
  const room = scene.stage.clientHeight - 16 - scene.head.offsetHeight - gap;
  const fit = Math.max(0.6, Math.min(1, room / scene.show.offsetHeight));
  scene.el.style.setProperty("--fit", String(fit));
  scene.el.classList.toggle("tight", fit < 1);
  let drop = 0;
  if (scene.art) {
    // Measured without the drawing's own scaling.
    const stage = scene.stage.getBoundingClientRect();
    const was = scene.art.style.transform;
    scene.art.style.transform = "none";
    const box = scene.art.getBoundingClientRect();
    scene.art.style.transform = was;
    const view = scene.art.viewBox.baseVal;
    const tall = Math.min(box.height, (box.width * view.height) / view.width);
    // With room, the heading sits inside the drawing; on a narrow screen, under it.
    drop = matchMedia("(max-width: 48rem)").matches
      ? box.top -
        stage.top +
        box.height / 2 +
        tall / 2 -
        stage.height / 2 +
        scene.head.offsetHeight * 0.54 +
        12
      : tall * Number(scene.art.dataset.drop ?? 0);
  }
  const lift = (scene.show.offsetHeight * fit + gap) / 2 + drop;
  scene.el.style.setProperty("--lift", `${lift}px`);
}

/** Splits a sentence into words that light up as it is read. */
function wordsOf(sentence: HTMLElement): HTMLElement[] {
  const words = (sentence.textContent ?? "").trim().split(/\s+/);
  const spans = words.map((word) => {
    const span = document.createElement("span");
    span.textContent = word;
    return span;
  });
  sentence.replaceChildren(...spans.flatMap((span, i) => (i === 0 ? [span] : [" ", span])));
  return spans;
}

/** Builds the page's motion for the current layout; returns what removes it. */
function build(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
  let scenes: Scene[] = [];
  const backdrops: Backdrop[] = [];
  /** Text this script rewrites, to put back as it was. */
  const rewritten = new Map<HTMLElement, string>();
  /** The scene being kept on screen to finish, if any. */
  let keeper: Scene | null = null;

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
            scene.entered = false;
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

    for (const scene of scenes) {
      // A scene kept on screen covers what follows; those scenes wait their turn.
      if (keeper && scene.index > keeper.index) continue;
      const go = settled ? put : send;
      const box = scene.el.getBoundingClientRect();
      const pinnedAt = Number.parseFloat(getComputedStyle(scene.stage).top) || 0;
      const tall = scene.stage.offsetHeight;
      const pinEnds = scrollY + box.bottom - pinnedAt - tall;
      const past = scrollY > pinEnds + 1;

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

      if (past) {
        const unfinished = scene.timeline.time() < scene.endAt - 0.02;
        if (scene.entered && unfinished && !keeper && !settled) keep(scene, box);
        else if (onScreen) go(scene, scene.endAt);
        else {
          put(scene, scene.endAt);
          scene.entered = false;
        }
        continue;
      }
      if (!onScreen) {
        // Not reached yet: at its very beginning.
        put(scene, 0);
        scene.entered = false;
        continue;
      }
      scene.entered = true;
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
      const typed = el.querySelector<HTMLElement>("[data-type]");
      if (typed) rewritten.set(typed, typed.textContent ?? "");
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

    // A sentence lights up word by word as it is read.
    const sentence = root.querySelector<HTMLElement>("[data-words]");
    if (sentence) {
      rewritten.set(sentence, sentence.textContent ?? "");
      const words = wordsOf(sentence);
      ScrollTrigger.create({
        trigger: sentence,
        start: "top 80%",
        end: "bottom 55%",
        onUpdate: (self) => {
          const lit = Math.ceil(self.progress * words.length);
          words.forEach((word, i) => {
            word.classList.toggle("lit", i < lit);
          });
        },
      });
    }

    // Tiles and facts rise in as they arrive, each a little after its
    // neighbour, and their small icons draw themselves.
    const arrivals = [...root.querySelectorAll<HTMLElement>("[data-reveal] > *")];
    gsap.set(arrivals, { autoAlpha: 0, y: 32, scale: 0.97 });
    gsap.set(root.querySelectorAll("svg.ico path"), { drawSVG: "0%", autoAlpha: 0 });
    gsap.set(root.querySelectorAll(".ico.os svg"), { autoAlpha: 0, y: 6 });
    ScrollTrigger.batch(arrivals, {
      start: "top 98%",
      once: true,
      onEnter: (batch) => {
        gsap.to(batch, {
          autoAlpha: 1,
          y: 0,
          scale: 1,
          duration: 0.8,
          ease: "power3.out",
          stagger: 0.08,
        });
        batch.forEach((item, i) => {
          item.classList.add("seen");
          const icon = item.querySelectorAll("svg.ico path");
          const delay = 0.35 + i * 0.08;
          gsap.to(icon, { autoAlpha: 1, duration: 0.12, delay });
          gsap.to(icon, { drawSVG: "100%", duration: 1, ease: "power2.inOut", delay });
          gsap.to(item.querySelectorAll(".ico.os svg"), {
            autoAlpha: 1,
            y: 0,
            duration: 0.6,
            delay: delay + 0.1,
            stagger: 0.15,
          });
        });
      },
    });

    // Figures count to their value when they arrive.
    for (const figure of root.querySelectorAll<HTMLElement>("[data-count]")) {
      rewritten.set(figure, figure.textContent ?? "");
      const to = Number(figure.dataset.count);
      const count = { value: to >= 20 ? to * 6 : 1 };
      ScrollTrigger.create({
        trigger: figure,
        start: "top 90%",
        once: true,
        onEnter: () => {
          gsap.to(count, {
            value: to,
            duration: 1.4,
            ease: "power2.inOut",
            onUpdate: () => {
              figure.textContent = String(Math.round(count.value));
            },
          });
        },
      });
    }
  }, root);

  for (const scene of scenes) measure(scene);
  place(true);
  addEventListener("scroll", request, { passive: true, signal: stop.signal });
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
      for (const pick of scene.el.querySelectorAll(".picked")) pick.classList.remove("picked");
    }
    for (const backdrop of backdrops) gsap.killTweensOf(backdrop.timeline);
    context.revert();
    for (const [el, text] of rewritten) {
      el.textContent = text;
      el.classList.remove("caret");
    }
    for (const item of root.querySelectorAll(".seen")) item.classList.remove("seen");
  };
}

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
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
  if (document.fonts.status !== "loaded") {
    void document.fonts.ready.then(() => {
      if (!stop.signal.aborted) rebuild();
    });
  }
  return () => {
    stop.abort();
    unbuild();
    root.classList.remove("motion");
  };
}
