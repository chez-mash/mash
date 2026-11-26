// Whether the reader is scrolling down the page right now. On a phone
// Mashy's circle floats in the corner with no column of its own, so it
// tucks toward the screen's edge while the reader scrolls down through what
// it would cover, and comes back as soon as the reader scrolls up, stops, or
// reaches the end. This only reports the scrolling, on the page's root
// element; the stylesheet decides where that matters and what must keep the
// circle in view.

/** Where the page is scrolled to, and how much of it there is. */
export interface ScrollPlace {
  readonly top: number;
  readonly viewport: number;
  readonly height: number;
}

/** How long the reader rests before the circle comes back, in milliseconds. */
export const SETTLE_MS = 700;

/**
 * Whether the reader is going down the page after a scroll from `from` to
 * `to`, given whether they were before. A jump of more than half a screen
 * at once is the page moving itself (a jump to a message, or older messages
 * loading above), not the reader, so it changes nothing.
 */
export function goingDown(was: boolean, from: number, to: ScrollPlace): boolean {
  if (to.top <= 0 || to.top + to.viewport >= to.height - 2) return false;
  const moved = to.top - from;
  if (moved === 0 || Math.abs(moved) > to.viewport / 2) return was;
  return moved > 0;
}

/** Reports scrolling down as `data-scrolling="down"` on `root` until stopped. */
export function watchScrolling(root: HTMLElement = document.documentElement): () => void {
  let last = window.scrollY;
  let down = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const show = (next: boolean): void => {
    down = next;
    if (next) root.dataset.scrolling = "down";
    else delete root.dataset.scrolling;
  };
  const scrolled = (): void => {
    const place = { top: window.scrollY, viewport: window.innerHeight, height: root.scrollHeight };
    show(goingDown(down, last, place));
    last = place.top;
    clearTimeout(timer);
    if (down) timer = setTimeout(() => show(false), SETTLE_MS);
  };
  window.addEventListener("scroll", scrolled, { passive: true });
  return () => {
    window.removeEventListener("scroll", scrolled);
    clearTimeout(timer);
    show(false);
  };
}
