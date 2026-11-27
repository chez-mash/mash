/**
 * The card rails' behaviour.
 *
 * A rail is a plain scroll container that snaps to its cards, so it works
 * without this script. The script shows the previous and next buttons, which
 * move one card and switch off at either end, makes the arrow keys move one
 * card while the rail has the focus, and marks each card `seen` once it has
 * come into view, so its picture can play in (the rail is marked `rail-live`
 * first; without the script nothing is hidden).
 */

type Stop = () => void;

/** Starts one rail; returns what stops it and shows every card again. */
function mountRail(rail: HTMLElement): Stop {
  const track = rail.querySelector<HTMLElement>("[data-rail-track]");
  const go = rail.querySelector<HTMLElement>("[data-rail-go]");
  if (!track || !go) return () => {};
  const buttons = [...go.querySelectorAll<HTMLButtonElement>("button")];
  /** One card and the gap after it, in pixels. */
  const step = () => {
    const [first, second] = track.children;
    if (!(first instanceof HTMLElement)) return track.clientWidth;
    return second instanceof HTMLElement ? second.offsetLeft - first.offsetLeft : first.offsetWidth;
  };
  const move = (by: number) => track.scrollBy({ left: by * step(), behavior: "smooth" });
  const refresh = () => {
    const end = track.scrollWidth - track.clientWidth;
    for (const button of buttons)
      button.disabled =
        button.dataset.step === "-1" ? track.scrollLeft <= 1 : track.scrollLeft >= end - 1;
  };
  const onClick = (event: MouseEvent) => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button[data-step]");
    if (button) move(Number(button.dataset.step));
  };
  const onKey = (event: KeyboardEvent) => {
    if (event.target !== track || (event.key !== "ArrowRight" && event.key !== "ArrowLeft")) return;
    event.preventDefault();
    move(event.key === "ArrowRight" ? 1 : -1);
  };
  // A card counts as seen when most of it is on screen, in the page and in the rail.
  const cards = [...track.children].filter(
    (card): card is HTMLElement => card instanceof HTMLElement,
  );
  const seen = new IntersectionObserver(
    (entries) => {
      for (const entry of entries)
        if (entry.isIntersecting) {
          entry.target.classList.add("seen");
          seen.unobserve(entry.target);
        }
    },
    { threshold: 0.6 },
  );
  for (const card of cards) seen.observe(card);
  rail.classList.add("rail-live");
  go.hidden = false;
  go.addEventListener("click", onClick);
  track.addEventListener("keydown", onKey);
  track.addEventListener("scroll", refresh, { passive: true });
  const sized = new ResizeObserver(refresh);
  sized.observe(track);
  refresh();
  return () => {
    rail.classList.remove("rail-live");
    for (const card of cards) card.classList.remove("seen");
    seen.disconnect();
    go.hidden = true;
    go.removeEventListener("click", onClick);
    track.removeEventListener("keydown", onKey);
    track.removeEventListener("scroll", refresh);
    sized.disconnect();
  };
}

/** Starts every rail in `root`; returns what stops them. */
export function mountRails(root: HTMLElement): Stop {
  const stops = [...root.querySelectorAll<HTMLElement>("[data-rail]")].map(mountRail);
  return () => {
    for (const stop of stops) stop();
  };
}
