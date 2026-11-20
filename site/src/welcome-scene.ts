/**
 * One pinned scene of the welcome page as a GSAP timeline, and the drawing
 * helpers the rest of the page shares. See welcome.ts for how scroll steers it.
 */
import { gsap } from "gsap";

/** How much later each stroke of a drawing starts than the one before, in strokes. */
const STAGGER = 0.12;
/** Seconds a whole drawing takes to draw. */
const DRAW_SECONDS = 1;
/** How long a finished drawing has the screen to itself before its panel rises. */
const HOLD_SECONDS = 0.5;
/** Letters a second for a typed query. */
const TYPING_RATE = 22;
/** How faint a drawing is behind its panel. */
const FAINT = 0.13;

export interface Scene {
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

/** The strokes of a drawing, and the shapes that fill in once they are drawn. */
export function parts(art: Element | null): { strokes: SVGElement[]; dots: SVGElement[] } {
  if (!art) return { strokes: [], dots: [] };
  return {
    strokes: [...art.querySelectorAll<SVGElement>("path:not(.dot, .glint)")],
    dots: [...art.querySelectorAll<SVGElement>("circle, .dot, .glint")],
  };
}

/** Appends drawing the strokes one after another, then the filled shapes. */
export function draw(
  timeline: gsap.core.Timeline,
  strokes: SVGElement[],
  dots: SVGElement[],
): void {
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
export function sceneIn(el: HTMLElement, index: number): Scene | undefined {
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
  // What the panel holds that is not a step arrives with it: facts rise in
  // one after another, their icons draw, figures count up, a sentence lights.
  const up = timeline.duration();
  const arrivals = [...el.querySelectorAll<HTMLElement>("[data-reveal] > *")];
  if (arrivals.length > 0) {
    gsap.set(arrivals, { autoAlpha: 0, y: 24 });
    const icons = el.querySelectorAll("[data-reveal] svg.ico path");
    gsap.set(icons, { drawSVG: "0%", autoAlpha: 0 });
    timeline.to(
      arrivals,
      { autoAlpha: 1, y: 0, duration: 0.5, ease: "power2.out", stagger: 0.08 },
      up - 0.25,
    );
    timeline.to(icons, { autoAlpha: 1, duration: 0.1, ease: "none" }, up);
    timeline.to(icons, { drawSVG: "100%", duration: 0.8, ease: "power2.inOut" }, up);
  }
  const figures = [...el.querySelectorAll<HTMLElement>("[data-count]")].map((figure) => {
    const to = Number(figure.dataset.count);
    const count = { value: to >= 20 ? to * 6 : 1 };
    timeline.to(count, { value: to, duration: 1.1, ease: "power2.inOut" }, up - 0.25);
    return { figure, count };
  });
  const sentence = el.querySelector<HTMLElement>("[data-words]");
  const words = sentence ? wordsOf(sentence) : [];
  const reading = { lit: 0 };
  if (words.length > 0) {
    timeline.to(reading, { lit: words.length, duration: words.length * 0.05, ease: "none" }, up);
  }
  const panelAt = timeline.duration();

  // A step may bring its own parts in one by one (`data-each`).
  const each = (item: HTMLElement): HTMLElement[] =>
    item.dataset.each === undefined
      ? []
      : [...item.children].filter((c) => c instanceof HTMLElement);
  for (const item of stepped) {
    gsap.set(each(item), { autoAlpha: 0, y: 18 });
    gsap.set(item.querySelectorAll("[data-each] svg.ico path"), { drawSVG: "0%", autoAlpha: 0 });
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
      const from = at + (before.length > 0 ? 0.2 : 0);
      timeline.to(item, { autoAlpha: 1, y: 0, scale: 1, duration: 0.45, ease: "power2.out" }, from);
      each(item).forEach((part, i) => {
        const when = from + 0.15 + i * 0.45;
        timeline.to(part, { autoAlpha: 1, y: 0, duration: 0.5, ease: "power2.out" }, when);
        const icon = part.querySelectorAll("svg.ico path");
        timeline.to(icon, { autoAlpha: 1, duration: 0.08, ease: "none" }, when + 0.1);
        timeline.to(icon, { drawSVG: "100%", duration: 0.6, ease: "power2.inOut" }, when + 0.1);
      });
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
    for (const { figure, count } of figures) figure.textContent = String(Math.round(count.value));
    words.forEach((word, i) => {
      word.classList.toggle("lit", i < Math.ceil(reading.lit - 0.001));
    });
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
export function measure(scene: Scene): void {
  const gap = Number.parseFloat(getComputedStyle(scene.stage).rowGap) || 0;
  const room = scene.stage.clientHeight - 16 - scene.head.offsetHeight - gap;
  const fit = Math.max(0.6, Math.min(1, room / scene.show.offsetHeight));
  scene.el.style.setProperty("--fit", String(fit));
  scene.el.classList.toggle("tight", fit < 1);
  // The heading is enlarged while it has the screen to itself, but never past
  // the screen's edges: a long one-line heading grows only as far as it fits.
  const narrow = matchMedia("(max-width: 48rem)").matches;
  const style = getComputedStyle(scene.stage);
  const inner =
    scene.stage.clientWidth -
    Number.parseFloat(style.paddingLeft) -
    Number.parseFloat(style.paddingRight);
  const wide = scene.head.querySelector("h2")?.offsetWidth ?? scene.head.offsetWidth;
  const grow = Math.max(0, Math.min(narrow ? 0.08 : 0.22, inner / wide - 1));
  scene.head.style.setProperty("--grow", String(grow));
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
    drop = narrow
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
export function wordsOf(sentence: HTMLElement): HTMLElement[] {
  const words = (sentence.textContent ?? "").trim().split(/\s+/);
  const spans = words.map((word) => {
    const span = document.createElement("span");
    span.textContent = word;
    return span;
  });
  sentence.replaceChildren(...spans.flatMap((span, i) => (i === 0 ? [span] : [" ", span])));
  return spans;
}
