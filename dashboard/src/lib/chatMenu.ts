// What a chat's right-click menu offers, wherever a chat is listed: open it,
// copy a link back to it, and, for a chat of this machine whose agent can
// resume it, copy the command that does. The same things the chat page and
// the quick panel's actions offer (ChatHeader.svelte, QuickActions.svelte).

import type { MenuEntry } from "./contextMenu.svelte";
import { resumeCommand } from "./openIn";
import { shareLink } from "./shareLink";
import type { Agent } from "./types";

export interface MenuChat {
  readonly agent: Agent;
  readonly id: string;
}

async function copy(text: string): Promise<void> {
  try {
    await navigator.clipboard?.writeText(text);
  } catch {
    // Nothing to tell: the menu has closed and the clipboard is unchanged.
  }
}

/** The menu for `chat`, which opens at `href`; `local` when it is this machine's. */
export function chatMenu(chat: MenuChat, href: string, local: boolean): MenuEntry[] {
  const entries: MenuEntry[] = [
    {
      title: "Open",
      run: () => {
        location.hash = href.replace(/^#/, "");
      },
    },
    "separator",
    {
      title: "Copy Link",
      run: () => void copy(shareLink(href, `${location.origin}${location.pathname}`)),
    },
  ];
  const command = local ? resumeCommand(chat.agent, chat.id) : undefined;
  if (command) entries.push({ title: "Copy Resume Command", run: () => void copy(command) });
  return entries;
}
