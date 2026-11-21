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
/** Reading time at full visibility, including when a scene is finishing quickly. */
const CAPTION_SECONDS = 1.1;
const ILLUSTRATION_SECONDS = 1.2;
/** How faint a drawing is behind its panel. */
const FAINT = 0.13;

export interface Scene {
  readonly index: number;
  readonly el: HTMLElement;
  readonly stage: HTMLElement;
  /** The heading, where the scene has one. */
  readonly head: HTMLElement | null;
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

/** Brings in the cards and draws their line icons as the panel settles. */
function revealIconTiles(timeline: gsap.core.Timeline, show: HTMLElement, panelAt: number): void {
  const tiles = show.querySelectorAll<HTMLElement>("[data-icon-tile]");
  tiles.forEach((tile, index) => {
    const when = panelAt - 0.08 + index * 0.12;
    const strokes = tile.querySelectorAll<SVGPathElement>("svg.ico path");
    gsap.set(tile, { opacity: 0, y: 12 });
    timeline.to(tile, { opacity: 1, y: 0, duration: 0.28, ease: "power2.out" }, when);
    if (strokes.length > 0) {
      gsap.set(strokes, { drawSVG: "0%", autoAlpha: 0 });
      timeline.to(strokes, { autoAlpha: 1, duration: 0.08 }, when + 0.12);
      timeline.to(strokes, { drawSVG: "100%", duration: 0.55, ease: "power2.inOut" }, when + 0.12);
    }
  });
}

/** Builds one scene's whole performance as a paused timeline. */
export function sceneIn(el: HTMLElement, index: number): Scene | undefined {
  const stage = el.querySelector<HTMLElement>(".stage");
  const head = el.querySelector<HTMLElement>(".head");
  const show = el.querySelector<HTMLElement>(".show");
  if (!stage || !show) return undefined;
  const captions = [...show.querySelectorAll<HTMLElement>(".caps > li")];
  const steps = Math.max(1, captions.length);
  const art = el.querySelector<SVGSVGElement>("svg.art");
  const strength = Number(art?.dataset.strength ?? 0.5);
  const { strokes, dots } = parts(art);
  const timeline = gsap.timeline({ paused: true });

  // The drawing, then a moment with the heading alone.
  gsap.set(el, { "--t": 0, "--art": strength, "--arrival-side": index % 2 === 0 ? -1 : 1 });
  draw(timeline, strokes, dots);
  const drawnAt = timeline.duration();
  if (head) timeline.to({}, { duration: HOLD_SECONDS });
  const titledAt = timeline.duration();

  // The heading makes way for the panel, and the drawing recedes behind it.
  timeline.to(el, { "--t": 1, "--art": FAINT, duration: 0.55, ease: "power2.inOut" });
  const panelAt = timeline.duration();

  if (captions.length > 0) gsap.set(captions, { autoAlpha: 0, y: 26, scale: 0.97 });
  const stepAt = captions.map((caption, step) => {
    const at = timeline.duration();
    const previous = captions[step - 1];
    if (previous) {
      timeline.to(
        previous,
        { autoAlpha: 0, y: -26, scale: 0.97, duration: 0.3, ease: "power2.in" },
        at,
      );
    }
    timeline.to(
      caption,
      { autoAlpha: 1, y: 0, scale: 1, duration: 0.45, ease: "power2.out" },
      at + (previous ? 0.2 : 0),
    );
    timeline.to({}, { duration: CAPTION_SECONDS });
    return timeline.duration();
  });
  if (stepAt.length === 0) {
    timeline.to({}, { duration: ILLUSTRATION_SECONDS });
    stepAt.push(timeline.duration());
  }
  revealIconTiles(timeline, show, panelAt);

  // The panel leaves, the drawing comes back to strength and is undrawn.
  timeline.to(el, { "--t": 0, "--art": strength, duration: 0.45, ease: "power2.inOut" });
  undraw(timeline, strokes, dots);
  const endAt = timeline.duration();

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
    kept: false,
  };
}

/** How far a centered, scaled item can travel while staying inside the stage. */
function entranceTravel(stageWidth: number, scaledWidth: number): number {
  return Math.max(0, Math.min(144, stageWidth * 0.18, (stageWidth - scaledWidth) / 2 - 4));
}

/**
 * Sizes a scene for the screen: a panel too tall is scaled down to fit, and the
 * heading and panel get room to move while staying inside the screen.
 */
export function measure(scene: Scene): void {
  const style = getComputedStyle(scene.stage);
  const gap = Number.parseFloat(style.rowGap) || 0;
  const above = scene.head ? scene.head.offsetHeight + gap : 0;
  const room = scene.stage.clientHeight - 16 - above;
  const fit = Math.max(0.6, Math.min(1, room / scene.show.offsetHeight));
  scene.el.style.setProperty("--fit", String(fit));
  scene.el.classList.toggle("tight", fit < 1);
  // The heading is enlarged while it has the screen to itself, but never past
  // the screen's edges: a long one-line heading grows only as far as it fits.
  const narrow = matchMedia("(max-width: 48rem)").matches;
  const stageWidth = scene.stage.clientWidth;
  const inner =
    stageWidth - Number.parseFloat(style.paddingLeft) - Number.parseFloat(style.paddingRight);
  const wide = scene.head?.querySelector("h2")?.offsetWidth ?? inner;
  const grow = Math.max(0, Math.min(narrow ? 0.08 : 0.22, inner / wide - 1));
  scene.head?.style.setProperty("--grow", String(grow));
  // At --t: 0 the heading is enlarged and the panel is at 92% of its fitted
  // size. Their offsets both fall to zero as those scales settle.
  const showTravel = entranceTravel(stageWidth, scene.show.offsetWidth * 0.92 * fit);
  scene.el.style.setProperty("--show-travel", `${showTravel}px`);
  if (scene.head) {
    const headTravel = entranceTravel(stageWidth, scene.head.offsetWidth * (1 + grow));
    scene.el.style.setProperty("--head-travel", `${headTravel}px`);
  }
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
        (scene.head?.offsetHeight ?? 0) * 0.54 +
        12
      : tall * Number(scene.art.dataset.drop ?? 0);
  }
  const lift = (scene.show.offsetHeight * fit + gap) / 2 + drop;
  scene.el.style.setProperty("--lift", `${lift}px`);
}
