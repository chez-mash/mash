/**
 * The surface showcase's motion. The showcase changes surface by itself
 * (`surfaces.ts`); this plays each change it announces: the picture slides out
 * the way the change goes while the next slides in, the next picture's search
 * types itself and its rows rise, the chosen dock tile grows and lifts as the
 * last one settles, the tag glides over to it, and the caption fades up.
 *
 * The page is already in its final state when a change is announced, so each
 * part is animated from where it was and handed back to the stylesheet. A
 * change that arrives mid-way finishes the one before it at once.
 */
import { gsap } from "gsap";
import type { SurfaceChange } from "../surfaces";
import { all, rise, type Stop } from "./stage";

/** How far a picture travels sideways as it leaves or arrives, in pixels. */
const slide = 36;

/** Types an element's text in place; the untyped rest holds its space. Returns what restores it. */
function typeOut(timeline: gsap.core.Timeline, element: HTMLElement, at: number): Stop {
  const text = element.textContent ?? "";
  const said = document.createElement("span");
  const unsaid = document.createElement("span");
  unsaid.style.opacity = "0";
  unsaid.textContent = text;
  element.replaceChildren(said, unsaid);
  const typed = { count: 0 };
  timeline.to(
    typed,
    {
      count: text.length,
      duration: Math.min(0.7, 0.04 * text.length),
      ease: `steps(${text.length})`,
      onUpdate: () => {
        const count = Math.round(typed.count);
        said.textContent = text.slice(0, count);
        unsaid.textContent = text.slice(count);
      },
    },
    at,
  );
  return () => element.replaceChildren(text);
}

/** Plays the showcase's changes; returns what stops it. */
export function surfaces(root: HTMLElement): Stop {
  const tag = root.querySelector<HTMLElement>(".dock-tag");
  const part = (kind: string, id: string) =>
    root.querySelector<HTMLElement>(`${kind}[data-surface="${id}"]`);
  let running: gsap.core.Timeline | undefined;
  let finish: Stop = () => {};

  const onChange = (event: Event) => {
    const { from, to, direction } = (event as CustomEvent<SurfaceChange>).detail;
    running?.progress(1);
    const leaving = part(".surface", from);
    const arriving = part(".surface", to);
    const [last, chosen] = [part('[role="tab"]', from), part('[role="tab"]', to)];
    const caption = part(".caption", to);
    if (!leaving || !arriving || !last || !chosen) return;

    const restores: Stop[] = [];
    finish = () => {
      leaving.classList.remove("leaving");
      for (const restore of restores) restore();
      finish = () => {};
    };
    const timeline = gsap.timeline({ onComplete: () => finish() });
    running = timeline;

    leaving.classList.add("leaving");
    timeline.fromTo(
      leaving,
      { opacity: 1, x: 0 },
      {
        opacity: 0,
        x: -slide * direction,
        duration: 0.22,
        ease: "power2.in",
        clearProps: "opacity,transform",
      },
      0,
    );
    timeline.fromTo(
      arriving,
      { opacity: 0, x: slide * direction },
      { opacity: 1, x: 0, duration: 0.4, ease: "power3.out", clearProps: "opacity,transform" },
      0.14,
    );
    for (const typed of all(arriving, "[data-typed]")) restores.push(typeOut(timeline, typed, 0.3));
    rise(timeline, all(arriving, "[data-in]"), 0.4, { y: 8, duration: 0.3, stagger: 0.05 });

    // The dock: the stylesheet already draws the chosen tile grown and lifted.
    const lift = -0.5 * Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    const grown = { y: lift, scale: 1.18 };
    const flat = { y: 0, scale: 1 };
    timeline.fromTo(
      last,
      grown,
      { ...flat, duration: 0.25, ease: "power2.out", clearProps: "transform" },
      0,
    );
    timeline.fromTo(
      chosen,
      flat,
      { ...grown, duration: 0.45, ease: "back.out(2.2)", clearProps: "transform" },
      0,
    );
    if (tag) {
      const travelled = last.offsetLeft - chosen.offsetLeft;
      timeline.fromTo(
        tag,
        { x: travelled },
        { x: 0, duration: 0.35, ease: "power3.out", clearProps: "transform" },
        0,
      );
    }
    if (caption) rise(timeline, caption, 0.1, { y: 6, duration: 0.35 });
  };

  root.addEventListener("surfaces:change", onChange);
  return () => {
    root.removeEventListener("surfaces:change", onChange);
    running?.kill();
    finish();
    gsap.set(all(root, ".surface, .caption, .dock-tag, [role='tab'], [data-in]"), {
      clearProps: "opacity,transform",
    });
  };
}
