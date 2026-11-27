/**
 * Starts the welcome page: the plugin's step rail, the surface showcase, Mashy's examples, and the
 * page's motion.
 *
 * The motion (`motion.ts`, with GSAP) is fetched here, on this page only and
 * after the page has been drawn. Without this script the page is simply
 * shown; the stylesheet hides nothing.
 */

import { mountRails } from "./rails.ts";
import { mountSurfaces } from "./surfaces.ts";

/** Lets the hero show if the motion never takes over its entrance. */
function showHero(): void {
  for (const hero of document.querySelectorAll<HTMLElement>(".mash-hero[data-motion]"))
    delete hero.dataset.motion;
}

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement): () => void {
  const chats = [...root.querySelectorAll<HTMLElement>("[data-mashy-chat]")].map(mountMashyChat);
  const showcases = [...root.querySelectorAll<HTMLElement>("[data-surfaces]")].map(mountSurfaces);
  const rails = mountRails(root);
  let stopped = false;
  let stopMotion: (() => void) | undefined;
  import("./motion.ts")
    .then(({ mountMotion }) => {
      if (stopped) return;
      stopMotion = mountMotion(document);
      root.classList.add("reveal");
    })
    .catch(showHero);
  return () => {
    stopped = true;
    stopMotion?.();
    showHero();
    for (const stop of [...chats, ...showcases, rails]) stop();
    root.classList.remove("reveal");
  };
}

/** How long each Mashy example stays before the next. */
const exampleMs = 6000;

/** A change of Mashy's example, announced on the carousel as `mashy:change`. */
export interface ExampleChange {
  readonly from: number;
  readonly to: number;
  /** 1 when the change goes on to a later example, -1 when it goes back. */
  readonly direction: 1 | -1;
}

/** How far a finger travels sideways, in pixels, before it counts as a swipe. */
const swipe = 40;

/**
 * Shows Mashy's examples one at a time. They change by themselves while on
 * screen, unless the reader prefers less motion, and stop for good once the
 * reader picks one: with the pager, its arrow keys, or a swipe across the
 * window. Each change is announced for the motion to play (motion/mashy.ts);
 * the page is already in its final state by then. Returns what stops it and
 * shows every example again.
 */
function mountMashyChat(carousel: HTMLElement): () => void {
  const examples = [...carousel.querySelectorAll<HTMLElement>("[data-example]")];
  const pager = carousel.querySelector<HTMLElement>(".pager");
  const dots = [...carousel.querySelectorAll<HTMLButtonElement>("[data-example-to]")];
  const count = examples.length;
  if (!pager || count === 0) return () => {};
  let current = 1;
  let timer: number | undefined;
  let playing = !matchMedia("(prefers-reduced-motion: reduce)").matches;
  let onScreen = false;
  let paused = false;

  const show = (next: number, direction: 1 | -1 = 1) => {
    const from = current;
    current = ((next - 1 + count) % count) + 1;
    carousel.dataset.example = String(current);
    for (const example of examples)
      example.classList.toggle("on", Number(example.dataset.example) === current);
    for (const dot of dots) {
      if (Number(dot.dataset.exampleTo) === current) dot.setAttribute("aria-current", "true");
      else dot.removeAttribute("aria-current");
    }
    if (from !== current) {
      const detail: ExampleChange = { from, to: current, direction };
      carousel.dispatchEvent(new CustomEvent("mashy:change", { detail }));
    }
    schedule();
  };
  const schedule = () => {
    clearTimeout(timer);
    timer = undefined;
    if (!playing || !onScreen || paused) return;
    timer = window.setTimeout(() => show(current + 1), exampleMs);
  };
  const choose = (next: number, direction: 1 | -1) => {
    playing = false;
    show(next, direction);
  };
  const step = (direction: 1 | -1) => choose(current + direction, direction);

  const onClick = (event: MouseEvent) => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button");
    if (!button) return;
    const to = Number(button.dataset.exampleTo);
    if (button.dataset.exampleTo) choose(to, to < current ? -1 : 1);
    else if (button.dataset.go) step(button.dataset.go === "-1" ? -1 : 1);
  };
  // With the pager in focus the arrow keys go back and on.
  const onKey = (event: KeyboardEvent) => {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    step(event.key === "ArrowLeft" ? -1 : 1);
  };
  // A swipe across the window: to the left goes on, to the right goes back.
  let touched: { x: number; y: number } | undefined;
  const onTouchStart = (event: TouchEvent) => {
    const touch = event.touches.length === 1 ? event.touches[0] : undefined;
    touched = touch && { x: touch.clientX, y: touch.clientY };
  };
  const onTouchEnd = (event: TouchEvent) => {
    const touch = event.changedTouches[0];
    if (!touched || !touch) return;
    const [across, down] = [touch.clientX - touched.x, touch.clientY - touched.y];
    touched = undefined;
    if (Math.abs(across) >= swipe && Math.abs(across) > 1.5 * Math.abs(down))
      step(across < 0 ? 1 : -1);
  };
  const pause = () => {
    paused = true;
    schedule();
  };
  const resume = () => {
    paused = false;
    schedule();
  };
  const watch = new IntersectionObserver(([entry]) => {
    onScreen = entry?.isIntersecting ?? false;
    schedule();
  });

  pager.hidden = false;
  pager.addEventListener("click", onClick);
  pager.addEventListener("keydown", onKey);
  carousel.addEventListener("touchstart", onTouchStart, { passive: true });
  carousel.addEventListener("touchend", onTouchEnd);
  carousel.addEventListener("pointerenter", pause);
  carousel.addEventListener("pointerleave", resume);
  carousel.addEventListener("focusin", pause);
  carousel.addEventListener("focusout", resume);
  watch.observe(carousel);
  show(1);

  return () => {
    clearTimeout(timer);
    watch.disconnect();
    pager.removeEventListener("click", onClick);
    pager.removeEventListener("keydown", onKey);
    carousel.removeEventListener("touchstart", onTouchStart);
    carousel.removeEventListener("touchend", onTouchEnd);
    carousel.removeEventListener("pointerenter", pause);
    carousel.removeEventListener("pointerleave", resume);
    carousel.removeEventListener("focusin", pause);
    carousel.removeEventListener("focusout", resume);
    pager.hidden = true;
    delete carousel.dataset.example;
    for (const example of examples) example.classList.remove("on");
  };
}
