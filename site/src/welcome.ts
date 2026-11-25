/**
 * Brings each band of the welcome page in gently as it scrolls into view.
 *
 * Without this script, or when the reader prefers less motion, every band is
 * simply shown: the stylesheet only hides a band once the page has the
 * `reveal` class, and only when motion is welcome.
 */

/** Starts the page; returns what stops it again. */
export function mountWelcome(root: HTMLElement): () => void {
  const groups = [...root.querySelectorAll<HTMLElement>("[data-reveal]")];
  if (!("IntersectionObserver" in window)) return () => {};
  const seen = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add("shown");
        seen.unobserve(entry.target);
      }
    },
    { rootMargin: "0px 0px -8% 0px" },
  );
  // A band already on screen, or above it after a jump to an anchor, shows at once.
  for (const group of groups) {
    if (group.getBoundingClientRect().top < innerHeight) group.classList.add("shown");
    else seen.observe(group);
  }
  root.classList.add("reveal");
  const chats = [...root.querySelectorAll<HTMLElement>("[data-mashy-chat]")].map(mountMashyChat);
  return () => {
    for (const stop of chats) stop();
    seen.disconnect();
    root.classList.remove("reveal");
    for (const group of groups) group.classList.remove("shown");
  };
}

/** How long each Mashy example stays before the next. */
const exampleMs = 6000;

/**
 * Shows Mashy's examples one at a time. They change by themselves while on
 * screen, unless the reader prefers less motion, and stop for good once the
 * reader picks one. Returns what stops it and shows every example again.
 */
function mountMashyChat(carousel: HTMLElement): () => void {
  const examples = [...carousel.querySelectorAll<HTMLElement>("[data-example]")];
  const pager = carousel.querySelector<HTMLElement>(".pager");
  const dots = [...carousel.querySelectorAll<HTMLButtonElement>("[data-example-to]")];
  const count = examples.length;
  if (!pager || count === 0) return () => {};
  let current = 1;
  let timer: number | undefined;
  let playing = !matchMedia("(prefers-reduced-motion: reduce)").matches;
  let onScreen = false;
  let paused = false;

  const show = (next: number) => {
    current = ((next - 1 + count) % count) + 1;
    carousel.dataset.example = String(current);
    for (const example of examples)
      example.classList.toggle("on", Number(example.dataset.example) === current);
    for (const dot of dots) {
      if (Number(dot.dataset.exampleTo) === current) dot.setAttribute("aria-current", "true");
      else dot.removeAttribute("aria-current");
    }
    schedule();
  };
  const schedule = () => {
    clearTimeout(timer);
    timer = undefined;
    if (!playing || !onScreen || paused) return;
    timer = window.setTimeout(() => show(current + 1), exampleMs);
  };
  const choose = (next: number) => {
    playing = false;
    show(next);
  };

  const onClick = (event: MouseEvent) => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button");
    if (!button) return;
    if (button.dataset.exampleTo) choose(Number(button.dataset.exampleTo));
    else if (button.dataset.go) choose(current + Number(button.dataset.go));
  };
  const pause = () => {
    paused = true;
    schedule();
  };
  const resume = () => {
    paused = false;
    schedule();
  };
  const watch = new IntersectionObserver(([entry]) => {
    onScreen = entry?.isIntersecting ?? false;
    schedule();
  });

  pager.hidden = false;
  pager.addEventListener("click", onClick);
  carousel.addEventListener("pointerenter", pause);
  carousel.addEventListener("pointerleave", resume);
  carousel.addEventListener("focusin", pause);
  carousel.addEventListener("focusout", resume);
  watch.observe(carousel);
  show(1);

  return () => {
    clearTimeout(timer);
    watch.disconnect();
    pager.removeEventListener("click", onClick);
    carousel.removeEventListener("pointerenter", pause);
    carousel.removeEventListener("pointerleave", resume);
    carousel.removeEventListener("focusin", pause);
    carousel.removeEventListener("focusout", resume);
    pager.hidden = true;
    delete carousel.dataset.example;
    for (const example of examples) example.classList.remove("on");
  };
}
