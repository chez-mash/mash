/**
 * The sliding highlight of a segmented control. One element behind the
 * segments is moved and resized to the chosen segment's box, so a change of
 * choice slides the highlight across instead of jumping; the stylesheet
 * transitions its transform and width, and switches at once for a reader who
 * prefers less motion.
 *
 * The track carries `data-sliding` while the highlight is in use, and the
 * segments then leave their own raised background to it. Without it (before
 * the first measure, with nothing chosen, when the segments wrap onto several
 * rows, or when the chosen one is disabled and so dimmed) each segment draws
 * its own, and a change jumps as it did.
 */

/** As long as the stylesheet's transition, to tell a resize during a slide from one at rest. */
const slideMs = 200;

interface Box {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface Slide {
  /** Matches the chosen segment inside the track. */
  readonly chosen: string;
  /** The attributes whose change means another segment was chosen. */
  readonly watch: readonly string[];
  /**
   * A name to keep the highlight's place under when the control is redrawn
   * from scratch for each choice (a row of links), so the new one slides from
   * where the last one stood.
   */
  readonly remember?: string;
}

const remembered = new Map<string, Box>();

const sameBox = (a: Box, b: Box): boolean =>
  a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height;

/** The segment's box inside the track's padding edge, in the track's own pixels. */
function boxOf(track: HTMLElement, segment: HTMLElement): Box {
  const outer = track.getBoundingClientRect();
  const inner = segment.getBoundingClientRect();
  // A track drawn scaled (a dialog opening) still measures in its own pixels.
  // The layout width is a whole number, so less than a pixel apart is no scale.
  const laid = track.offsetWidth;
  const scale = laid > 0 && Math.abs(outer.width - laid) >= 1 ? outer.width / laid : 1;
  return {
    x: (inner.left - outer.left) / scale - track.clientLeft + track.scrollLeft,
    y: (inner.top - outer.top) / scale - track.clientTop + track.scrollTop,
    width: inner.width / scale,
    height: inner.height / scale,
  };
}

function draw(indicator: HTMLElement, box: Box): void {
  indicator.style.transform = `translate(${box.x}px, ${box.y}px)`;
  indicator.style.width = `${box.width}px`;
  indicator.style.height = `${box.height}px`;
}

/** Keeps `indicator` on the chosen segment of `track`; returns what stops it. */
export function slideIndicator(
  track: HTMLElement,
  indicator: HTMLElement,
  slide: Slide,
): () => void {
  let last: HTMLElement | null = null;
  let slidingUntil = 0;
  let from = slide.remember ? remembered.get(slide.remember) : undefined;

  const segments = () =>
    [...track.children].filter(
      (child): child is HTMLElement => child instanceof HTMLElement && child !== indicator,
    );
  const jump = (box: Box) => {
    indicator.style.transition = "none";
    draw(indicator, box);
    // Read back, so the next change starts from here.
    void indicator.offsetWidth;
    indicator.style.transition = "";
  };

  const place = () => {
    const all = segments();
    // Several chosen at once (a group that allows it) each keep their own background.
    const marked = track.querySelectorAll<HTMLElement>(slide.chosen);
    const chosen = marked.length === 1 ? (marked[0] ?? null) : null;
    const first = all[0];
    const oneRow =
      first !== undefined &&
      all.every((segment) => segment.offsetTop < first.offsetTop + first.offsetHeight);
    const dimmed = chosen?.matches(":disabled, [aria-disabled='true']") ?? false;
    if (!chosen || !oneRow || dimmed || chosen.offsetWidth === 0) {
      delete track.dataset.sliding;
      last = chosen;
      return;
    }
    const box = boxOf(track, chosen);
    const wasSliding = track.dataset.sliding !== undefined;
    const moved = wasSliding && last !== null && chosen !== last;
    const arrived = !wasSliding && from !== undefined && !sameBox(from, box);
    track.dataset.sliding = "";
    if (arrived && from) jump(from);
    if (moved || arrived) slidingUntil = performance.now() + slideMs;
    if (performance.now() < slidingUntil) draw(indicator, box);
    else jump(box);
    from = undefined;
    last = chosen;
    if (slide.remember) remembered.set(slide.remember, box);
  };

  const resized = new ResizeObserver(place);
  const observe = () => {
    resized.disconnect();
    resized.observe(track);
    for (const segment of segments()) resized.observe(segment);
  };
  const changed = new MutationObserver((records) => {
    if (records.some((record) => record.type === "childList")) observe();
    place();
  });
  changed.observe(track, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: [...slide.watch],
  });
  observe();
  place();

  return () => {
    resized.disconnect();
    changed.disconnect();
    delete track.dataset.sliding;
  };
}
