/** Mashy's scene: his answer types out, and he bobs while he is on screen. */
import { gsap } from "gsap";
import { all, below, loopWhileSeen, playOnce, rise, type Screen, type Stop } from "./stage";

/**
 * G. Mashy: his first answer types out in place and its quote follows, and on
 * a wide screen he bobs. The answer's whole text holds its space while it
 * types, so the window keeps its size. His other examples are left alone.
 */
export function mashy(band: HTMLElement, screen: Screen): Stop {
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
