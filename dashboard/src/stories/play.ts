// Steps a story takes after it has drawn (its `play` function).
import { screen, userEvent } from "storybook/test";

/**
 * A menu or dialog that is open from the start takes focus the moment it is
 * drawn, and a browser nobody has clicked in yet rings whatever a script
 * focuses. Opened with the pointer, as it is in the dashboard, the panel has
 * no ring, so the story lets go of the focus to show it that way.
 */
export async function openedByPointer(): Promise<void> {
  if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
}

/** Rests the pointer on a control until its hint shows; hints are drawn outside the story's root. */
export async function hint(control: HTMLElement, text: string): Promise<void> {
  await userEvent.hover(control);
  await screen.findByText(text);
}
