/** The two scenes built around the app icon: the hero at the top and the closing card. */
import { gsap } from "gsap";
import { all, below, loopWhileSeen, playOnce, rise, type Screen } from "./stage";

/** A. The hero: the app icon settles, the headline rises a line at a time, the rest follows. */
export function hero(root: HTMLElement, screen: Screen, entrance: boolean): void {
  if (entrance) {
    const y = screen.wide ? 24 : 16;
    const timeline = gsap.timeline();
    timeline.fromTo(
      all(root, ".hero-icon"),
      { opacity: 0, scale: 0.92 },
      { opacity: 1, scale: 1, duration: 0.6, ease: "power3.out", clearProps: "opacity,transform" },
      0,
    );
    timeline.fromTo(
      all(root, ".hero-icon .halo"),
      { opacity: 0, scale: 0.8 },
      { opacity: 1, scale: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity,transform" },
      0.15,
    );
    const lines = all(root, "h1 > span");
    const lineRise = { at: 0.12, duration: 0.5, stagger: 0.1 };
    rise(timeline, lines, lineRise.at, { y, ease: "power3.out", ...lineRise });
    // The stroke under the last line is drawn from the left, a beat after that
    // line has settled. It is on the same timeline at its own time, so nothing
    // else waits for it. 310 is the dash that hides the whole line (Hero.astro).
    const settled = lineRise.at + (lines.length - 1) * lineRise.stagger + lineRise.duration;
    timeline.fromTo(
      all<SVGPathElement>(root, "h1 .draw"),
      { strokeDashoffset: 310 },
      { strokeDashoffset: 0, duration: 0.7, ease: "power2.inOut", clearProps: "strokeDashoffset" },
      settled + 0.15,
    );
    rise(timeline, all(root, ".tagline"), 0.38, { y: 16, duration: 0.44 });
    rise(timeline, all(root, ".actions, .download-note"), 0.46, { y: 16, duration: 0.44 });
  }
  if (!screen.wide) return;
  // The streaks drift a few pixels for as long as the hero is on screen.
  const drift = all(root, ".streak").map((streak, index) =>
    gsap
      .fromTo(
        streak,
        { x: -14, y: 5 },
        { x: 14, y: -5, duration: 9 + index * 1.4, ease: "sine.inOut", yoyo: true, repeat: -1 },
      )
      .progress(0.5),
  );
  loopWhileSeen(root, drift);
}

/** H. The closing card: the app icon rises with the hero's glow. */
export function closing(card: HTMLElement, screen: Screen): void {
  if (!below(card)) return;
  const timeline = gsap.timeline({ paused: true });
  timeline.fromTo(
    all(card, ".close-icon"),
    { opacity: 0, y: screen.wide ? 28 : 16, scale: 0.92 },
    {
      opacity: 1,
      y: 0,
      scale: 1,
      duration: 0.7,
      ease: "power3.out",
      clearProps: "opacity,transform",
    },
    0,
  );
  timeline.fromTo(
    all(card, ".close-icon .halo"),
    { opacity: 0, scale: 0.8 },
    { opacity: 1, scale: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity,transform" },
    0.15,
  );
  timeline.fromTo(
    all(card, ".close-glow"),
    { opacity: 0 },
    { opacity: 1, duration: 0.75, ease: "power2.out", clearProps: "opacity" },
    0.15,
  );
  rise(timeline, all(card, "h2"), 0.2, { y: 16, duration: 0.5, ease: "power3.out" });
  rise(timeline, all(card, ".actions"), 0.32, { y: 16, duration: 0.5 });
  playOnce(card, timeline, "top 82%");
}
