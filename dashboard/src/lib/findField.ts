// The top bar's search field can be asked for from anywhere, such as the
// rail's Search row: the bar shows the field (on a phone it is hidden until
// asked for) and moves focus into it.

/** The window event the top bar listens for. */
export const FIND_EVENT = "mash:find";

/** Shows the search field and focuses it. */
export function askForSearch(): void {
  dispatchEvent(new Event(FIND_EVENT));
}
