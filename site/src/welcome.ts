/**
 * Starts the welcome page: the surface showcase, Mashy's examples, and the
 * page's motion.
 *
 * The motion (`motion.ts`, with GSAP) is fetched here, on this page only and
 * after the page has been drawn. Without this script the page is simply
 * shown; the stylesheet hides nothing.
 */

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
    for (const stop of [...chats, ...showcases]) stop();
    root.classList.remove("reveal");
  };
}

/** How long each Mashy example stays before the next. */
const exampleMs = 6000;

/**
 * Shows Mashy's examples one at a time. They change by themselves while on
 * screen, unless the reader prefers less motion, and stop for good once the
 * reader picks one. Returns what stops it and shows every example again.
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

  const show = (next: number) => {
    current = ((next - 1 + count) % count) + 1;
    carousel.dataset.example = String(current);
    for (const example of examples)
      example.classList.toggle("on", Number(example.dataset.example) === current);
    for (const dot of dots) {
      if (Number(dot.dataset.exampleTo) === current) dot.setAttribute("aria-current", "true");
      else dot.removeAttribute("aria-current");
    }
    schedule();
  };
  const schedule = () => {
    clearTimeout(timer);
    timer = undefined;
    if (!playing || !onScreen || paused) return;
    timer = window.setTimeout(() => show(current + 1), exampleMs);
  };
  const choose = (next: number) => {
    playing = false;
    show(next);
  };

  const onClick = (event: MouseEvent) => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button");
    if (!button) return;
    if (button.dataset.exampleTo) choose(Number(button.dataset.exampleTo));
    else if (button.dataset.go) choose(current + Number(button.dataset.go));
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
    carousel.removeEventListener("pointerenter", pause);
    carousel.removeEventListener("pointerleave", resume);
    carousel.removeEventListener("focusin", pause);
    carousel.removeEventListener("focusout", resume);
    pager.hidden = true;
    delete carousel.dataset.example;
    for (const example of examples) example.classList.remove("on");
  };
}
