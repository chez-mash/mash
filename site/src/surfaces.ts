/**
 * The surface showcase's behaviour: the dock under the picture is a tablist,
 * and choosing a tab shows that surface and its caption.
 *
 * As served, the first surface shows and each dock button is a link to its
 * guide. This script takes the links over as tabs: a click, the arrow keys,
 * Home and End, or a sideways swipe on the picture choose a surface. The
 * surfaces also change by themselves every few seconds while the picture is
 * on screen and the reader is not pointing at or focused in the band. A
 * choice by the reader stops that for a while; it picks up again once the
 * reader has left the showcase alone. A `data-hold` attribute on the showcase
 * stops it for as long as it is there, which is how a picture is taken of it. Each change is announced to the page's motion with
 * a `surfaces:change` event; without the motion the change is simply made.
 */

/** What a `surfaces:change` event carries: the surfaces' ids and the way the change goes. */
export interface SurfaceChange {
  from: string;
  to: string;
  /** 1 when the next surface comes from the right, -1 from the left. */
  direction: 1 | -1;
}

/** How long each surface stays before the next shows by itself. */
const stayMs = 5000;
/** How long the showcase waits after the reader's last choice before it changes by itself again. */
const restMs = 20_000;
/** How far a finger travels sideways, in pixels, before it counts as a swipe. */
const swipePx = 40;

/** Starts the showcase in `root`; returns what stops it and shows the first surface again. */
export function mountSurfaces(root: HTMLElement): () => void {
  const tabs = [...root.querySelectorAll<HTMLAnchorElement>('[role="tab"]')];
  const stage = root.querySelector<HTMLElement>(".stage");
  const area = root.querySelector<HTMLElement>(".dock-area");
  const tag = root.querySelector<HTMLElement>("[data-dock-tag]");
  const count = tabs.length;
  if (!stage || !area || !tag || count < 2) return () => {};
  const parts = (id: string | undefined) =>
    root.querySelectorAll<HTMLElement>(
      `.surface[data-surface="${id}"], .caption[data-surface="${id}"]`,
    );

  let current = 0;
  let timer: number | undefined;
  let rest: number | undefined;
  let playing = true;
  let seen = false;
  let held = false;

  /** Draws surface `index` as the chosen one, without announcing it. */
  const draw = (index: number) => {
    for (const [at, tab] of tabs.entries()) {
      const on = at === index;
      tab.setAttribute("aria-selected", String(on));
      tab.tabIndex = on ? 0 : -1;
      for (const part of parts(tab.dataset.surface)) part.classList.toggle("on", on);
    }
    area.style.setProperty("--at", String(index));
    tag.textContent = tabs[index]?.getAttribute("aria-label") ?? "";
    current = index;
  };
  const schedule = () => {
    clearTimeout(timer);
    timer = undefined;
    if (playing && seen && !held && !document.hidden && !root.hasAttribute("data-hold"))
      timer = window.setTimeout(() => {
        if (!root.hasAttribute("data-hold")) show(current + 1, 1);
      }, stayMs);
  };
  const show = (next: number, direction: 1 | -1) => {
    const index = (next + count) % count;
    if (index !== current) {
      const detail: SurfaceChange = {
        from: tabs[current]?.dataset.surface ?? "",
        to: tabs[index]?.dataset.surface ?? "",
        direction,
      };
      draw(index);
      root.dispatchEvent(new CustomEvent("surfaces:change", { detail }));
    }
    schedule();
  };
  /** A choice by the reader, which rests the changes the page makes by itself. */
  const choose = (next: number, direction: 1 | -1, focus = false) => {
    playing = false;
    clearTimeout(rest);
    rest = window.setTimeout(() => {
      playing = true;
      schedule();
    }, restMs);
    show(next, direction);
    if (focus) tabs[current]?.focus();
  };

  const onClick = (event: MouseEvent) => {
    const tab = (event.target as Element).closest<HTMLAnchorElement>('[role="tab"]');
    // A click that asks for a new tab or window still follows the link.
    if (!tab || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    const index = tabs.indexOf(tab);
    choose(index, index < current ? -1 : 1);
  };
  const onKey = (event: KeyboardEvent) => {
    const index = tabs.indexOf(event.target as HTMLAnchorElement);
    if (index < 0) return;
    if (event.key === "ArrowRight") choose(current + 1, 1, true);
    else if (event.key === "ArrowLeft") choose(current - 1, -1, true);
    else if (event.key === "Home") choose(0, -1, true);
    else if (event.key === "End") choose(count - 1, 1, true);
    else if (event.key === " " || event.key === "Enter") choose(index, index < current ? -1 : 1);
    else return;
    event.preventDefault();
  };

  let from: { id: number; x: number; y: number } | undefined;
  const onDown = (event: PointerEvent) => {
    from =
      event.pointerType === "mouse"
        ? undefined
        : { id: event.pointerId, x: event.clientX, y: event.clientY };
  };
  const onUp = (event: PointerEvent) => {
    if (from?.id !== event.pointerId) return;
    const across = event.clientX - from.x;
    const down = event.clientY - from.y;
    from = undefined;
    if (Math.abs(across) < swipePx || Math.abs(across) < 1.5 * Math.abs(down)) return;
    // The picture follows the finger: a swipe to the left brings the next one.
    if (across < 0) choose(current + 1, 1);
    else choose(current - 1, -1);
  };
  const onCancel = () => {
    from = undefined;
  };

  const hold = () => {
    held = true;
    schedule();
  };
  const release = () => {
    held = false;
    schedule();
  };
  const onEnter = (event: PointerEvent) => {
    if (event.pointerType === "mouse") hold();
  };
  // Focus from the keyboard holds the surface; the focus a click leaves behind does not.
  const onFocus = (event: FocusEvent) => {
    if ((event.target as Element).matches(":focus-visible")) hold();
  };
  const watch = new IntersectionObserver(
    ([entry]) => {
      seen = entry?.isIntersecting ?? false;
      schedule();
    },
    { threshold: 0.5 },
  );

  const dock = tabs[0]?.parentElement;
  dock?.addEventListener("click", onClick);
  dock?.addEventListener("keydown", onKey);
  stage.addEventListener("pointerdown", onDown);
  stage.addEventListener("pointerup", onUp);
  stage.addEventListener("pointercancel", onCancel);
  root.addEventListener("pointerenter", onEnter);
  root.addEventListener("pointerleave", release);
  root.addEventListener("focusin", onFocus);
  root.addEventListener("focusout", release);
  document.addEventListener("visibilitychange", schedule);
  watch.observe(stage);
  root.dataset.surfaces = "live";
  draw(0);

  return () => {
    clearTimeout(timer);
    clearTimeout(rest);
    watch.disconnect();
    dock?.removeEventListener("click", onClick);
    dock?.removeEventListener("keydown", onKey);
    stage.removeEventListener("pointerdown", onDown);
    stage.removeEventListener("pointerup", onUp);
    stage.removeEventListener("pointercancel", onCancel);
    root.removeEventListener("pointerenter", onEnter);
    root.removeEventListener("pointerleave", release);
    root.removeEventListener("focusin", onFocus);
    root.removeEventListener("focusout", release);
    document.removeEventListener("visibilitychange", schedule);
    draw(0);
    for (const tab of tabs) tab.removeAttribute("tabindex");
    root.dataset.surfaces = "";
  };
}
