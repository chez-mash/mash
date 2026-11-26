/**
 * Mashy's scene: his answer types out, he bobs while he is on screen, and a
 * change of example slides the last one out as the next slides in.
 */
import { gsap } from "gsap";
import type { ExampleChange } from "../welcome";
import { all, below, loopWhileSeen, playOnce, rise, type Screen, type Stop } from "./stage";

/** How far an example travels sideways as it leaves or arrives, in pixels. */
const slide = 28;

/**
 * Plays the carousel's changes (`welcome.ts` announces each, the page already
 * in its final state): the example left behind slides away the way the change
 * goes and fades, and the next slides in from the other side, with the surface
 * showcase's easing. The examples share one grid cell, so the window keeps its
 * size. A change that arrives mid-way finishes the one before it at once, so
 * two never stay on screen. Returns what stops it.
 */
function examples(carousel: HTMLElement): Stop {
  const part = (example: number) =>
    carousel.querySelector<HTMLElement>(`.example[data-example="${example}"]`);
  let running: gsap.core.Timeline | undefined;
  const onChange = (event: Event) => {
    const { from, to, direction } = (event as CustomEvent<ExampleChange>).detail;
    running?.progress(1);
    const leaving = part(from);
    const arriving = part(to);
    if (!leaving || !arriving) return;
    const timeline = gsap.timeline();
    running = timeline;
    // The stylesheet has already hidden the one left behind; it stays in view while it goes.
    timeline.fromTo(
      leaving,
      { visibility: "visible", opacity: 1, x: 0 },
      {
        opacity: 0,
        x: -slide * direction,
        duration: 0.2,
        ease: "power2.in",
        clearProps: "visibility,opacity,transform",
      },
      0,
    );
    timeline.fromTo(
      arriving,
      { opacity: 0, x: slide * direction },
      { opacity: 1, x: 0, duration: 0.32, ease: "power3.out", clearProps: "opacity,transform" },
      0.1,
    );
  };
  carousel.addEventListener("mashy:change", onChange);
  return () => {
    carousel.removeEventListener("mashy:change", onChange);
    running?.kill();
    gsap.set(all(carousel, ".example"), { clearProps: "visibility,opacity,transform" });
  };
}

/**
 * G. Mashy: his first answer types out in place and its quote follows, and on
 * a wide screen he bobs. The answer's whole text holds its space while it
 * types, so the window keeps its size. His other examples are not typed.
 */
export function mashy(band: HTMLElement, screen: Screen): Stop {
  const face = band.querySelector<HTMLElement>(".mashy-face");
  if (face && screen.wide) {
    const bob = gsap
      .fromTo(face, { y: -2 }, { y: 2, duration: 1.5, ease: "sine.inOut", yoyo: true, repeat: -1 })
      .progress(0.5);
    loopWhileSeen(face, [bob]);
  }
  const carousel = band.querySelector<HTMLElement>("[data-mashy-chat]");
  const stopExamples = carousel ? examples(carousel) : () => {};
  const picture = band.querySelector<HTMLElement>(".mashy-window");
  const example = band.querySelector<HTMLElement>('.example[data-example="1"]');
  const answer = example?.querySelector<HTMLElement>(".answer");
  const sentence = answer?.querySelector<HTMLElement>(":scope > div > p");
  if (!picture || !answer || !sentence || !below(picture)) return stopExamples;

  const timeline = gsap.timeline({ paused: true });
  if (!screen.wide) {
    rise(timeline, answer, 0, { y: 12, duration: 0.45 });
    playOnce(picture, timeline);
    return stopExamples;
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
  return () => {
    restore();
    stopExamples();
  };
}
