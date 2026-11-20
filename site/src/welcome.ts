/**
 * Plays the welcome page as the reader scrolls: each scene pins under the
 * header, its drawing draws itself, its heading makes way for a panel, and the
 * panel's steps arrive one after another. Drawings and panels keep a pace of
 * their own, so a fast scroll cannot skip them.
 *
 * Without this script the page shows everything, statically.
 */

/** How much later each stroke of a drawing starts than the one before, in strokes. */
const STAGGER = 0.12;
/** The quickest a whole drawing may be drawn, and undrawn, in seconds. */
const DRAW_SECONDS = 1;
const UNDRAW_SECONDS = 1 / 3;
/** How long a finished drawing has the screen to itself before its panel rises. */
const HOLD_MS = 500;
/** The fastest the shown position may advance, in steps a second. */
const STEPS_PER_SECOND = 1.5;
/** Letters a second for a typed query. */
const TYPING_RATE = 22;
/** Where, in steps, the heading starts to make way and where the panel is fully up. */
const PANEL_FROM = 0.45;
const PANEL_UP = 0.8;

const clamp = (x: number): number => Math.min(1, Math.max(0, x));

/** Eased progress of `x` from `from` to `to`. */
function smooth(from: number, to: number, x: number): number {
  const t = clamp((x - from) / (to - from));
  return t * t * (3 - 2 * t);
}

interface Drawing {
  readonly el: HTMLElement;
  readonly art: SVGSVGElement | null;
  readonly strokes: readonly SVGElement[];
  readonly dots: readonly SVGElement[];
}

interface Scene extends Drawing {
  readonly steps: number;
  readonly stage: HTMLElement;
  readonly head: HTMLElement;
  readonly show: HTMLElement;
  readonly stepped: readonly HTMLElement[];
  readonly typed: HTMLElement | null;
  readonly query: string;
  /** The position shown, in steps, trailing the scroll; unset while off screen. */
  shown: number | undefined;
  /** The share of the drawing drawn so far; unset while off screen. */
  drawn: number | undefined;
  chars: number;
  /** When the drawing was last completed, or 0 while it is incomplete. */
  doneAt: number;
}

function drawingIn(el: HTMLElement): Drawing {
  return {
    el,
    art: el.querySelector<SVGSVGElement>("svg.art"),
    strokes: [...el.querySelectorAll<SVGElement>(".art path:not(.dot, .glint)")],
    dots: [...el.querySelectorAll<SVGElement>(".art circle, .art .dot, .art .glint")],
  };
}

/** Draws a share of a drawing: strokes in turn, filled shapes once the last is done. */
function trace(drawing: Drawing, amount: number): void {
  const whole = 1 + (drawing.strokes.length - 1) * STAGGER;
  const drawn = amount * whole;
  drawing.strokes.forEach((stroke, i) => {
    const part = clamp(drawn - i * STAGGER);
    stroke.style.strokeDashoffset = String(1 - part);
    stroke.style.opacity = part > 0 ? "1" : "0";
  });
  for (const dot of drawing.dots) dot.style.opacity = drawn >= whole - 0.02 ? "1" : "0";
}

function sceneIn(el: HTMLElement): Scene | undefined {
  const stage = el.querySelector<HTMLElement>(".stage");
  const head = el.querySelector<HTMLElement>(".head");
  const show = el.querySelector<HTMLElement>(".show");
  const steps = Number(el.dataset.steps);
  if (!stage || !head || !show || !Number.isFinite(steps) || steps < 1) return undefined;
  el.style.setProperty("--n", String(steps));
  const typed = el.querySelector<HTMLElement>("[data-type]");
  return {
    ...drawingIn(el),
    steps,
    stage,
    head,
    show,
    stepped: [...el.querySelectorAll<HTMLElement>("[data-step]")],
    typed,
    query: typed?.textContent ?? "",
    shown: undefined,
    drawn: undefined,
    chars: 0,
    doneAt: 0,
  };
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

/** Places a scene's steps and captions for the position shown. */
function place(scene: Scene, shown: number): void {
  for (const item of scene.stepped) {
    const step = Number(item.dataset.step);
    const x = shown - 1 - step;
    const arrive = smooth(-0.2, 0.3, x);
    const parent = item.parentElement;
    if (parent?.classList.contains("grow")) {
      // A thread: each turn takes its room as it arrives, pushing older ones
      // up, and suggestions fold away once one of them has been picked.
      const until = item.dataset.until === undefined ? undefined : Number(item.dataset.until);
      const leave = until === undefined ? 0 : smooth(-0.25, 0.15, shown - 1 - until);
      const here = arrive * (1 - leave);
      item.style.opacity = String(here);
      item.style.maxHeight = `${here * item.scrollHeight}px`;
      item.style.marginTop = `${here * 12}px`;
      item.style.transform = `translateY(${(1 - arrive) * 14}px)`;
      if (until !== undefined) {
        item.querySelector(".pick")?.classList.toggle("picked", shown - 1 - until > -0.7);
      }
      continue;
    }
    // Elsewhere neighbours cross-fade: one leaves as the next arrives.
    const swaps = parent?.matches(".caps, .swap") === true && step < scene.steps - 1;
    const leave = swaps ? smooth(0.5, 0.95, x) : 0;
    const here = arrive * (1 - leave);
    item.style.opacity = String(here);
    item.style.transform = `translateY(${(1 - arrive) * 26 - leave * 26}px) scale(${0.97 + 0.03 * here})`;
  }
}

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement, hero: HTMLElement | null): () => void {
  const stop = new AbortController();
  root.classList.add("motion");

  const scenes = [...root.querySelectorAll<HTMLElement>(".scene")].flatMap((el) => {
    const scene = sceneIn(el);
    return scene ? [scene] : [];
  });
  const backdrops = [...root.querySelectorAll<HTMLElement>(".backdrop")].map(drawingIn);
  for (const backdrop of backdrops) {
    const strength = backdrop.art?.dataset.strength;
    if (strength) backdrop.el.style.setProperty("--art", strength);
  }
  const sentence = root.querySelector<HTMLElement>("[data-words]");
  const words = sentence ? wordsOf(sentence) : [];

  let last = performance.now();
  let queued = 0;
  const request = (): void => {
    if (queued === 0) queued = requestAnimationFrame(frame);
  };

  function frame(now: number): void {
    queued = 0;
    let more = false;
    const vh = innerHeight;
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;

    if (hero) {
      const gone = clamp(scrollY / (hero.offsetHeight * 0.7));
      hero.style.opacity = String(1 - gone);
      hero.style.transform = `translateY(${gone * -32}px) scale(${1 - gone * 0.06})`;
    }

    if (sentence) {
      const box = sentence.getBoundingClientRect();
      const read = clamp((vh * 0.8 - box.top) / (box.height + vh * 0.25));
      const lit = Math.ceil(read * words.length);
      words.forEach((word, i) => {
        word.classList.toggle("lit", i < lit);
      });
    }

    for (const backdrop of backdrops) {
      const box = backdrop.el.getBoundingClientRect();
      if (box.bottom < 0 || box.top > vh) continue;
      trace(backdrop, clamp((vh * 0.95 - box.top) / Math.min(vh * 0.85, box.height)));
    }

    for (const scene of scenes) {
      const box = scene.el.getBoundingClientRect();
      if (box.bottom < -vh || box.top > 2 * vh) {
        // Well out of sight: forget where it was, and leave the drawing undrawn.
        if (scene.drawn !== undefined) trace(scene, 0);
        scene.shown = undefined;
        scene.drawn = undefined;
        scene.chars = 0;
        scene.doneAt = 0;
        continue;
      }
      // The stage pins under the site's header for the scene's extra height: a
      // part and a half for the heading alone, then one part per step.
      const pinnedAt = Number.parseFloat(getComputedStyle(scene.stage).top) || 0;
      const tall = scene.stage.offsetHeight;
      const enter = clamp((vh - box.top) / vh);
      const exit = clamp((pinnedAt + tall - box.bottom) / tall);
      const raw = clamp((pinnedAt - box.top) / (box.height - tall)) * (scene.steps + 1.5);
      const position = raw < 1.5 ? raw / 1.5 : raw - 0.5;
      scene.el.style.setProperty(
        "--o",
        String(smooth(0.35, 0.95, enter) * (1 - smooth(0.1, 0.7, exit))),
      );

      // The drawing takes its own time, however fast the scroll.
      const wanted = clamp((enter - 0.25) / 0.75) * 0.55 + clamp(raw / 0.3) * 0.45;
      scene.drawn =
        scene.drawn === undefined
          ? wanted
          : scene.drawn +
            Math.max(-dt / UNDRAW_SECONDS, Math.min(dt / DRAW_SECONDS, wanted - scene.drawn));
      if (Math.abs(wanted - scene.drawn) > 0.002) more = true;
      if (scene.drawn > 0.995) scene.doneAt ||= now;
      else scene.doneAt = 0;

      // What is shown trails the scroll at a limited pace. The panel waits for
      // the finished drawing, and its first step for a typed query's last letter.
      const held = scene.art !== null && !(scene.doneAt > 0 && now - scene.doneAt > HOLD_MS);
      let goal = held ? Math.min(position, PANEL_FROM) : position;
      if (held && position > PANEL_FROM) more = true;
      if (scene.typed) {
        const up = scene.shown !== undefined && scene.shown >= 0.7;
        scene.chars = up ? Math.min(scene.query.length, scene.chars + dt * TYPING_RATE) : 0;
        if (scene.chars < scene.query.length) {
          if (goal > PANEL_UP) more = true;
          goal = Math.min(goal, PANEL_UP);
        }
        const count = Math.floor(scene.chars);
        scene.typed.textContent = scene.query.slice(0, count);
        scene.typed.classList.toggle("caret", count < scene.query.length);
      }
      if (scene.shown === undefined) scene.shown = goal;
      else if (goal > scene.shown) {
        scene.shown += Math.min((goal - scene.shown) * 0.12, dt * STEPS_PER_SECOND);
      } else scene.shown += (goal - scene.shown) * 0.2;
      if (Math.abs(goal - scene.shown) > 0.002) more = true;
      else scene.shown = goal;

      scene.el.style.setProperty("--t", String(smooth(PANEL_FROM, PANEL_UP, scene.shown)));
      if (scene.art) {
        // Strong behind the heading, faint behind the panel.
        const strength = Number(scene.art.dataset.strength ?? 0.5);
        const faded = strength - (strength - 0.13) * smooth(0.6, 1.1, scene.shown);
        scene.el.style.setProperty("--art", String(faded));
        trace(scene, scene.drawn);
      }
      place(scene, scene.shown);
    }
    if (more) request();
  }

  const remeasure = (): void => {
    for (const scene of scenes) measure(scene);
    request();
  };
  addEventListener("scroll", request, { passive: true, signal: stop.signal });
  addEventListener("resize", remeasure, { signal: stop.signal });
  void document.fonts.ready.then(() => {
    if (!stop.signal.aborted) remeasure();
  });
  remeasure();

  // Tiles and facts rise in as they arrive, each a little after its neighbour.
  const arrivals = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add("seen");
        arrivals.unobserve(entry.target);
      }
    },
    { threshold: 0.2 },
  );
  for (const item of root.querySelectorAll<HTMLElement>("[data-reveal] > *")) {
    const order = [...(item.parentElement?.children ?? [])].indexOf(item) % 6;
    item.classList.add("reveal");
    item.style.setProperty("--d", String(order));
    arrivals.observe(item);
  }

  // Figures count to their value when they arrive.
  const counted = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const figure = entry.target;
        if (!entry.isIntersecting || !(figure instanceof HTMLElement)) continue;
        counted.unobserve(figure);
        const to = Number(figure.dataset.count);
        const from = to >= 20 ? to * 6 : 1;
        const began = performance.now();
        const tick = (now: number): void => {
          if (stop.signal.aborted) return;
          const k = smooth(0, 1, (now - began) / 1400);
          figure.textContent = String(Math.round(from + (to - from) * k));
          if (k < 1) requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      }
    },
    { threshold: 0.6 },
  );
  for (const figure of root.querySelectorAll("[data-count]")) counted.observe(figure);

  return () => {
    stop.abort();
    arrivals.disconnect();
    counted.disconnect();
    if (queued !== 0) cancelAnimationFrame(queued);
    if (hero) {
      hero.style.opacity = "";
      hero.style.transform = "";
    }
  };
}
