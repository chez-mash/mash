// The arithmetic of reading a chat in stretches: joining the stretches that
// are loaded into what the reader sees, swapping the last one for the same
// stretch read again, and deciding what a reread says about the chat. Nothing
// here touches the page or the network.
import type { ChatPlace, ChatStretch, MessagePage } from "./api";
import { type PageUpdate, pageUpdate } from "./live";

/** The loaded stretches and the places they were read from, as TanStack keeps them. */
export interface LoadedStretches {
  readonly pages: readonly ChatStretch[];
  readonly pageParams: readonly ChatPlace[];
}

/**
 * Everything loaded, read as one page. Stretches may share a tool call cut
 * off by an edge, so a message is kept the first time it appears.
 */
export function joinStretches(pages: readonly ChatStretch[]): MessagePage | undefined {
  const first = pages[0];
  const last = pages.at(-1);
  if (!first || !last) return undefined;
  const seen = new Set<string>();
  const messages = pages
    .flatMap((stretch) => stretch.page.messages)
    .filter((message) => !seen.has(message.id) && seen.add(message.id));
  const total = last.total ?? first.total;
  return {
    messages,
    ...(total !== undefined ? { total } : {}),
    ...(first.page.before ? { before: first.page.before } : {}),
    ...(last.page.after ? { after: last.page.after } : {}),
  };
}

/** Where to read `stretch` again from, asking for `limit` messages. */
export function rereadPlace(stretch: ChatStretch, limit: number): ChatPlace {
  return { at: "offset", offset: stretch.start, limit };
}

/**
 * The loaded stretches with the last one replaced by `next`, the same
 * stretch read again, recorded as read from its own edges so a later
 * refetch asks for the same messages.
 */
export function withLastReplaced(
  data: LoadedStretches | undefined,
  next: ChatStretch,
): { pages: ChatStretch[]; pageParams: ChatPlace[] } | undefined {
  if (!data) return undefined;
  return {
    pages: [...data.pages.slice(0, -1), next],
    pageParams: [...data.pageParams.slice(0, -1), rereadPlace(next, next.end - next.start)],
  };
}

function totalled(stretch: ChatStretch): { messages: MessagePage["messages"]; total?: number } {
  return {
    messages: stretch.page.messages,
    ...(stretch.total !== undefined ? { total: stretch.total } : {}),
  };
}

/**
 * What rereading says about the chat. `earlier` are stretches before the last
 * that were read again as `reread`, in the same order; any that differs, or
 * did not come back, means text already shown changed. Otherwise the last
 * stretch as shown is compared with `next`, the same stretch read again.
 */
export function rereadUpdate(
  earlier: readonly ChatStretch[],
  reread: readonly (ChatStretch | undefined)[],
  last: ChatStretch,
  next: ChatStretch,
): PageUpdate {
  const same = (a: MessagePage, b: MessagePage): boolean =>
    JSON.stringify(a.messages) === JSON.stringify(b.messages);
  const differs = earlier.some((stretch, index) => {
    const again = reread[index];
    return !again || !same(stretch.page, again.page);
  });
  if (differs) return { kind: "rewritten" };
  return pageUpdate(totalled(last), totalled(next));
}
