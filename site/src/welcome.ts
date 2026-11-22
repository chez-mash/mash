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
  return () => {
    seen.disconnect();
    root.classList.remove("reveal");
    for (const group of groups) group.classList.remove("shown");
  };
}
