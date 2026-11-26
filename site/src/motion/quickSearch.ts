/**
 * The quick search panel's scene, the one scene that can pin, and the repair
 * for a page opened at a link to something under the pin.
 */
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { all, below, playOnce, type Screen, type Stop } from "./stage";

/** What an empty quick search field says, as the desktop app's does. */
const placeholder = "Search every chat";

/**
 * B. Quick search: the query types itself, the results arrive, and the
 * selection moves down two rows. On a wide screen the panel pins and the
 * scroll drives it; elsewhere it plays once. The list keeps its height
 * throughout, so the panel never changes size.
 */
export function quickSearch(band: HTMLElement, screen: Screen): Stop {
  const desk = band.querySelector<HTMLElement>(".desk");
  const panel = band.querySelector<HTMLElement>(".quick");
  const field = band.querySelector<HTMLElement>(".quick-q b");
  const rows = all(band, ".quick li");
  const [first, second] = rows;
  if (!desk || !panel || !field || !first || !second) return () => {};
  if (!screen.pins && !below(panel)) return () => {};

  const query = field.textContent ?? "";
  const typed = { count: 0 };
  const type = () => {
    const count = Math.round(typed.count);
    field.textContent = count === 0 ? placeholder : query.slice(0, count);
    field.classList.toggle("empty", count === 0);
  };
  // The selection is one highlight that slides between rows.
  const selection = document.createElement("i");
  selection.className = "quick-sel";
  selection.style.top = `${first.offsetTop}px`;
  const step = second.offsetTop - first.offsetTop;
  panel.prepend(selection);
  panel.classList.add("live");
  type();

  // Times are fractions of the pin when scrubbed and seconds when played.
  const at = screen.pins
    ? { type: 0, typing: 0.3, rows: 0.3, row: 0.08, apart: 0.03, down: [0.55, 0.7], move: 0.15 }
    : { type: 0.3, typing: 0.6, rows: 0.9, row: 0.2, apart: 0.06, down: [1.4, 1.65], move: 0.2 };
  const timeline = gsap.timeline({ paused: true });
  timeline.to(
    typed,
    { count: query.length, duration: at.typing, ease: `steps(${query.length})`, onUpdate: type },
    at.type,
  );
  timeline.fromTo(
    all(band, ".quick-h"),
    { opacity: 0 },
    { opacity: 1, duration: at.row, ease: "power2.out" },
    at.rows,
  );
  timeline.fromTo(
    rows,
    { opacity: 0, y: 8 },
    {
      opacity: 1,
      y: 0,
      duration: at.row,
      ease: "power2.out",
      stagger: at.apart,
      // A scrubbed row may be scrolled back, so only a played one is handed back.
      clearProps: screen.pins ? "" : "opacity,transform",
    },
    at.rows,
  );
  timeline.fromTo(
    selection,
    { opacity: 0 },
    { opacity: 1, duration: at.row, ease: "power2.out" },
    at.rows + at.apart,
  );
  timeline.to(selection, { y: step, duration: at.move, ease: "power2.out" }, at.down[0]);
  timeline.to(selection, { y: step * 2, duration: at.move, ease: "power2.out" }, at.down[1]);

  if (screen.pins) {
    // The last part of the pin holds the finished panel before it lets go.
    timeline.to({}, { duration: 0.15 }, 0.85);
    ScrollTrigger.create({
      trigger: desk,
      start: "top 12%",
      end: "+=150%",
      pin: true,
      // The wrapper is a flex column, where the spacing is off unless asked for.
      pinSpacing: true,
      // Measured before the reveals below it, whose places depend on this spacing.
      refreshPriority: 1,
      scrub: 0.4,
      animation: timeline,
    });
  } else {
    timeline.fromTo(
      panel,
      { opacity: 0, y: 16 },
      { opacity: 1, y: 0, duration: 0.4, ease: "power2.out", clearProps: "opacity,transform" },
      0,
    );
    playOnce(panel, timeline, "top 70%");
  }

  return () => {
    selection.remove();
    panel.classList.remove("live");
    field.classList.remove("empty");
    field.textContent = query;
  };
}

/**
 * The pinned panel adds its scroll distance to the page once the script runs.
 * A page opened at a link to something under it would be left short of it, so
 * go to the link's target again.
 */
export function keepAnchor(pinned: Element | null): void {
  if (!pinned || location.hash.length < 2) return;
  const target = document.getElementById(decodeURIComponent(location.hash.slice(1)));
  if (!target) return;
  if (pinned.compareDocumentPosition(target) & Node.DOCUMENT_POSITION_FOLLOWING) {
    ScrollTrigger.refresh();
    target.scrollIntoView();
  }
}
