/**
 * The card rails' behaviour, and the tabs that choose between rails.
 *
 * A rail is a plain scroll container that snaps to its cards, so it works
 * without this script. The script shows the previous and next buttons, which
 * move one card and switch off at either end, and makes the arrow keys move
 * one card while the rail has the focus.
 *
 * A group of rails under tabs shows every rail, one under the other, as
 * served. An inline script beside the group marks it live before it is drawn,
 * and from then on one rail shows at a time in the same place; this script
 * makes the tabs choose it (a click, the arrow keys, Home and End).
 */

type Stop = () => void;

/** Starts one rail's buttons and keys; returns what stops them. */
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
  go.hidden = false;
  go.addEventListener("click", onClick);
  track.addEventListener("keydown", onKey);
  track.addEventListener("scroll", refresh, { passive: true });
  const sized = new ResizeObserver(refresh);
  sized.observe(track);
  refresh();
  return () => {
    go.hidden = true;
    go.removeEventListener("click", onClick);
    track.removeEventListener("keydown", onKey);
    track.removeEventListener("scroll", refresh);
    sized.disconnect();
  };
}

/** Starts a group's tabs; returns what stops them and shows the first rail again. */
function mountTabs(group: HTMLElement): Stop {
  const tabs = [...group.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
  const panels = [...group.querySelectorAll<HTMLElement>('[role="tabpanel"]')];
  const show = (index: number, focus = false) => {
    const chosen = (index + tabs.length) % tabs.length;
    for (const [at, tab] of tabs.entries()) {
      const on = at === chosen;
      tab.setAttribute("aria-selected", String(on));
      tab.tabIndex = on ? 0 : -1;
      panels[at]?.classList.toggle("on", on);
      if (on && focus) tab.focus();
    }
  };
  const onClick = (event: MouseEvent) => {
    const tab = (event.target as Element).closest<HTMLButtonElement>('[role="tab"]');
    if (tab) show(tabs.indexOf(tab));
  };
  const onKey = (event: KeyboardEvent) => {
    const at = tabs.indexOf(event.target as HTMLButtonElement);
    if (at < 0) return;
    const to: Record<string, number> = {
      ArrowRight: at + 1,
      ArrowLeft: at - 1,
      Home: 0,
      End: tabs.length - 1,
    };
    const next = to[event.key];
    if (next === undefined) return;
    event.preventDefault();
    show(next, true);
  };
  const list = tabs[0]?.parentElement;
  list?.addEventListener("click", onClick);
  list?.addEventListener("keydown", onKey);
  show(0);
  return () => {
    list?.removeEventListener("click", onClick);
    list?.removeEventListener("keydown", onKey);
    show(0);
  };
}

/** Starts every rail and tab group in `root`; returns what stops them. */
export function mountRails(root: HTMLElement): Stop {
  const stops = [
    ...[...root.querySelectorAll<HTMLElement>("[data-rail]")].map(mountRail),
    ...[...root.querySelectorAll<HTMLElement>("[data-rail-tabs]")].map(mountTabs),
  ];
  return () => {
    for (const stop of stops) stop();
  };
}
