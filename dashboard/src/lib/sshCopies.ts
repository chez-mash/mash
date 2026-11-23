import type { ChatSummary } from "./types";

/**
 * The Claude desktop app keeps a copy of each session it runs over SSH on
 * the computer it runs on, under the same chat ID as the original on the
 * remote machine. When another machine has a chat with that ID from the same
 * agent, and it is not a copy itself, the original is listed and the copy is
 * left out, so a session appears once. A copy with no original here stays,
 * marked as having run over SSH. `known` adds originals from elsewhere, such
 * as the catalogue beside a page of search results.
 */
export function withoutCopies<T extends { readonly chat: ChatSummary } | ChatSummary>(
  items: readonly T[],
  known: readonly ChatSummary[] = [],
): T[] {
  const chatOf = (item: T): ChatSummary => ("chat" in item ? item.chat : item);
  const originals = new Map<string, Set<string>>();
  for (const chat of [...items.map(chatOf), ...known]) {
    if (chat.sshCopy) continue;
    const key = `${chat.agent}\u0000${chat.id}`;
    originals.set(key, (originals.get(key) ?? new Set()).add(chat.machine));
  }
  const hasOriginal = (chat: ChatSummary): boolean =>
    [...(originals.get(`${chat.agent}\u0000${chat.id}`) ?? [])].some(
      (machine) => machine !== chat.machine,
    );
  return items.filter((item) => !(chatOf(item).sshCopy && hasOriginal(chatOf(item))));
}
