/**
 * What the landing page's scenes share: the screen variant a scene is built
 * for, and the few moves most of them make. A scene's entrance is prepared
 * only while the scene is still under the fold, so nothing on screen jumps.
 */
import type { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

type Timeline = gsap.core.Timeline;
export type Stop = () => void;

/** Which variant of each scene this screen gets. */
export interface Screen {
  /** A wide screen with a mouse: longer rises, the two loops, the slide and the typing. */
  wide: boolean;
  /** Wide and tall enough to hold the quick search panel while it pins. */
  pins: boolean;
}

export const all = <T extends Element = HTMLElement>(scope: ParentNode, selector: string): T[] => [
  ...scope.querySelectorAll<T>(selector),
];

/** Whether the element is still under the fold, so its entrance would be seen. */
export const below = (element: Element): boolean =>
  element.getBoundingClientRect().top >= innerHeight;

/** Fades targets up from `y` pixels lower, then hands them back to the stylesheet. */
export function rise(
  timeline: Timeline,
  targets: gsap.TweenTarget,
  at: number,
  vars: { y: number; duration: number; ease?: string; stagger?: number },
): void {
  timeline.fromTo(
    targets,
    { opacity: 0, y: vars.y },
    {
      opacity: 1,
      y: 0,
      duration: vars.duration,
      ease: vars.ease ?? "power2.out",
      stagger: vars.stagger ?? 0,
      clearProps: "opacity,transform",
    },
    at,
  );
}

/** Plays a paused timeline once, when its trigger reaches `start`. */
export function playOnce(trigger: Element, timeline: Timeline, start = "top 78%"): void {
  ScrollTrigger.create({ trigger, start, once: true, onEnter: () => timeline.play() });
}

/** Runs a loop only while its trigger is on screen. */
export function loopWhileSeen(trigger: Element, loops: gsap.core.Tween[]): void {
  for (const loop of loops) loop.pause();
  ScrollTrigger.create({
    trigger,
    start: "top bottom",
    end: "bottom top",
    onToggle: (self) => {
      for (const loop of loops) loop.paused(!self.isActive);
    },
  });
}
