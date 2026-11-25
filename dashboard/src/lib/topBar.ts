/**
 * Keeps `--top-h` on the document at the sticky top bar's height, so panes
 * that stay in view under it (the search preview, the list beside a chat)
 * start where it ends. Returns the function that stops following it.
 */
export function followTopBar(): () => void {
  const top = document.getElementById("top");
  if (!top || typeof ResizeObserver === "undefined") return () => {};
  const root = document.documentElement;
  const observer = new ResizeObserver(() => {
    root.style.setProperty("--top-h", `${top.offsetHeight}px`);
  });
  observer.observe(top);
  return () => observer.disconnect();
}
