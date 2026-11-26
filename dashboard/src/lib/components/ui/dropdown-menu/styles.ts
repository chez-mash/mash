// What every menu shares: the floating panel and its rows. A menu that needs
// a different size or ring passes classes, which replace the ones they
// conflict with.

/** The panel: a raised surface with a strong hairline. */
export const panel =
  "z-15 overflow-y-auto rounded-(--radius-6) border-(length:--stroke-strong) border-solid border-(--line-strong) bg-(--surface-raised) p-1 text-(--text-strong)";

/** A row: a finger-sized line that takes the hover tint under the pointer or the keyboard. */
export const row =
  "flex min-h-11 w-full cursor-pointer select-none items-center gap-2 rounded-(--radius-4) px-2.5 text-left text-[13.5px] [font-weight:var(--weight-label)] -outline-offset-2 data-highlighted:bg-(--surface-hover) data-disabled:cursor-default data-disabled:opacity-50";

/** A chosen row of a radio or checkbox menu. */
export const chosen =
  "data-[state=checked]:bg-(--selected-bg) data-[state=checked]:text-(--selected-text)";
